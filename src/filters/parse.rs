use super::{CompareOperator, Error, Expr, Value};
use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use std::str::FromStr;

/// Parses an OData v4 `$filter` expression string into an `Expr` AST.
///
/// # Arguments
///
/// * `query` - A string slice that holds the filter expression.
///
/// # Returns
///
/// A result containing the parsed `Expr` on success, or an `Error` on failure.
///
/// # Examples
///
/// ```
/// use odata_params::filters::parse_str;
///
/// let filter = "name eq 'John' and isActive eq true";
/// let result = parse_str(filter).expect("valid filter tree");
/// ```
pub fn parse_str(query: impl AsRef<str>) -> Result<Expr, Error> {
    match odata_filter::parse_str(query.as_ref()) {
        Ok(expr) => expr,
        Err(_error) => Err(Error::Parsing),
    }
}

enum AfterValueExpr {
    Compare(CompareOperator, Box<Expr>),
    In(Vec<Expr>),
    End,
}

peg::parser! {
    /// Parses OData v4 `$filter` expressions.
    grammar odata_filter() for str {
        use super::{Expr, CompareOperator, Value, Error};

        /// Entry point for parsing a filter expression string.
        pub(super) rule parse_str() -> Result<Expr, Error>
            = filter()

        /// Parses a filter expression.
        rule filter() -> Result<Expr, Error>
            = "not" _ e:filter() { Ok(Expr::Not(Box::new(e?))) }
            / l:any_expr() _ "or" _ r:filter() { Ok(Expr::Or(Box::new(l?), Box::new(r?))) }
            / l:any_expr() _ "and" _ r:filter() { Ok(Expr::And(Box::new(l?), Box::new(r?))) }
            / any_expr()

        /// Parses any expression, including grouped expressions and value expressions.
        rule any_expr() -> Result<Expr, Error>
            = "(" _ e:filter() _ ")" { e }
            / l:value_expr() _ r:after_value_expr() { Ok(match r? {
                AfterValueExpr::Compare(op, r) => Expr::Compare(Box::new(l?), op, r),
                AfterValueExpr::In(r) => Expr::In(Box::new(l?), r),
                AfterValueExpr::End => l?,
            }) }

        /// Parses an expression that comes after a value.
        rule after_value_expr() -> Result<AfterValueExpr, Error>
            = op:comparison_op() _ r:value_expr() { Ok(AfterValueExpr::Compare(op, Box::new(r?))) }
            / "in" _ "(" _ r:filter_list() _ ")" { Ok(AfterValueExpr::In(r?)) }
            / { Ok(AfterValueExpr::End) }

        /// Parses a value expression, which can be a function call, a value, or an identifier.
        rule value_expr() -> Result<Expr, Error>
            = function_call()
            / v:value() { Ok(Expr::Value(v?)) }
            / i:identifier() { Ok(Expr::Identifier(i)) }

        /// Parses a comparison operator.
        rule comparison_op() -> CompareOperator
            = "eq" { CompareOperator::Equal }
            / "ne" { CompareOperator::NotEqual }
            / "gt" { CompareOperator::GreaterThan }
            / "ge" { CompareOperator::GreaterOrEqual }
            / "lt" { CompareOperator::LessThan }
            / "le" { CompareOperator::LessOrEqual }

        /// Parses a function call with a name and arguments.
        rule function_call() -> Result<Expr, Error>
            = f:identifier() _ "(" _ l:filter_list() _ ")" { Ok(Expr::Function(f, l?)) }

        /// Parses an identifier.
        rule identifier() -> String
            = s:$(['a'..='z'|'A'..='Z'|'_']['a'..='z'|'A'..='Z'|'_'|'0'..='9']+) { s.to_string() }

        /// Parses a value, which can be a string, datetime, date, time, number, boolean, or null.
        rule value() -> Result<Value, Error>
            = string_value()
            / datetime_value()
            / date_value()
            / time_value()
            / number_value()
            / v:bool_value() { Ok(v) }
            / v:null_value() { Ok(v) }

        /// Parses a boolean value.
        rule bool_value() -> Value
            = ['t'|'T']['r'|'R']['u'|'U']['e'|'E'] { Value::Bool(true) }
            / ['f'|'F']['a'|'A']['l'|'L']['s'|'S']['e'|'E'] { Value::Bool(false) }

        /// Parses a numeric value.
        rule number_value() -> Result<Value, Error>
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) { Ok(Value::Number(BigDecimal::from_str(n).map_err(|_| Error::ParsingNumber)?)) }

        /// Parses a time value in the format `HH:MM:SS` or `HH:MM`.
        rule time() -> Result<NaiveTime, Error>
            = t:$($(['0'..='9']*<2>) ":" $(['0'..='9']*<2>) ":" $(['0'..='9']*<2>)) { NaiveTime::parse_from_str(t, "%H:%M:%S").map_err(|_| Error::ParsingTime) }
            / t:$($(['0'..='9']*<2>) ":" $(['0'..='9']*<2>)) { NaiveTime::parse_from_str(t, "%H:%M").map_err(|_| Error::ParsingTime) }

        /// Parses a time value.
        rule time_value() -> Result<Value, Error>
            = t:time() { Ok(Value::Time(t?)) }

        /// Parses a date value in the format `YYYY-MM-DD`.
        rule date() -> Result<NaiveDate, Error>
            = d:$($(['0'..='9']*<4>) "-" $(['0'..='9']*<2>) "-" $(['0'..='9']*<2>)) { NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| Error::ParsingDate) }

        /// Parses a date value.
        rule date_value() -> Result<Value, Error>
            = d:date() { Ok(Value::Date(d?)) }

        /// Parses a named timezone.
        rule timezone_name() -> Result<chrono_tz::Tz, Error>
            = z:$(['a'..='z'|'A'..='Z'|'-'|'_'|'/'|'+']['a'..='z'|'A'..='Z'|'-'|'_'|'/'|'+'|'0'..='9']+) { z.parse::<chrono_tz::Tz>().map_err(|_| Error::ParsingTimeZoneNamed) }

        /// Parses a timezone offset.
        rule timezone_offset() -> Result<FixedOffset, Error>
            = "Z" { "+0000".parse().map_err(|_| Error::ParsingTimeZone) }
            / z:$($(['-'|'+']) $(['0'..='9']*<2>) ":"? $(['0'..='9']*<2>)) { z.parse().map_err(|_| Error::ParsingTimeZone) }
            / z:$($(['-'|'+']) $(['0'..='9']*<2>)) { format!("{z}00").parse().map_err(|_| Error::ParsingTimeZone) }

        /// Parses a datetime value in the format `YYYY-MM-DDTHH:MM:SSZ` or `YYYY-MM-DDTHH:MM:SS+01:00`.
        rule datetime() -> Result<DateTime<Utc>, Error>
            = d:date() "T" t:time() z:timezone_offset() { Ok(d?.and_time(t?).and_local_timezone(z?).earliest().ok_or(Error::ParsingDateTime)?.to_utc()) }
            / d:date() "T" t:time() z:timezone_name() { Ok(d?.and_time(t?).and_local_timezone(z?).earliest().ok_or(Error::ParsingDateTime)?.to_utc()) }

        /// Parses a datetime value.
        rule datetime_value() -> Result<Value, Error>
            = dt:datetime() { Ok(Value::DateTime(dt?)) }

        /// Parses a string value enclosed in single quotes.
        rule string_value() -> Result<Value, Error>
            = "'" s:quote_escaped_string_content()* "'" { Ok(Value::String(s.into_iter().collect::<Result<Vec<_>, _>>()?.into_iter().collect())) }

        rule quote_escaped_string_content() -> Result<char, Error>
            = r"\" e:escape_character() { e }
            / c:[^'\''] { Ok(c) }

        rule escape_character() -> Result<char, Error>
            = "'" { Ok('\'') }
            / "n" { Ok('\n') }
            / "r" { Ok('\r') }
            / "t" { Ok('\t') }
            / r"\" { Ok('\\') }
            / "u" sequence:$(['0'..='9' | 'a'..='f' | 'A'..='F']+) {
                if sequence.len() > 8 { Err(Error::ParsingUnicodeCodePoint) }
                else { u32::from_str_radix(sequence, 16).ok().and_then(char::from_u32).ok_or(Error::ParsingUnicodeCodePoint) }
            }

        /// Parses a null value.
        rule null_value() -> Value
            = ['n'|'N']['u'|'U']['l'|'L']['l'|'L'] { Value::Null }

        /// Parses a list of value expressions separated by commas.
        rule value_list() -> Result<Vec<Expr>, Error>
            = v:value_expr() ** ( _ "," _ ) { v.into_iter().collect() }

        /// Parses a list of filter expressions separated by commas.
        rule filter_list() -> Result<Vec<Expr>, Error>
            = v:filter() ** ( _ "," _ ) { v.into_iter().collect() }

        /// Matches zero or more whitespace characters.
        rule _()
            = [' '|'\t'|'\n'|'\r']*
    }
}
