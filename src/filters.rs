use bigdecimal::BigDecimal;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use std::str::FromStr;

pub fn parse_str(query: impl AsRef<str>) -> Result<Expr, Error> {
    match odata_filter::parse_str(query.as_ref()) {
        Ok(expr) => expr,
        Err(_error) => Err(Error::Parsing),
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Parsing,
    ParsingNumber,
    ParsingDate,
    ParsingTime,
    ParsingDateTime,
    ParsingTimeZone,
    ParsingTimeZoneNamed,
}

impl std::error::Error for Error {}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Compare(Box<Expr>, CompareOperator, Box<Expr>),
    In(Box<Expr>, Vec<Expr>),
    Not(Box<Expr>),
    Function(String, Vec<Expr>),
    Identifier(String),
    Value(Value),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompareOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Null,
    Bool(bool),
    Number(BigDecimal),
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
    Time(NaiveTime),
    String(String),
}

enum AfterValueExpr {
    Compare(CompareOperator, Box<Expr>),
    In(Vec<Expr>),
    End,
}

peg::parser! {
    grammar odata_filter() for str {
        use super::{Expr, CompareOperator, Value, Error};

        pub(super) rule parse_str() -> Result<Expr, Error>
            = filter()

        rule filter() -> Result<Expr, Error>
            = "not" _ e:filter() { Ok(Expr::Not(Box::new(e?))) }
            / l:any_expr() _ "or" _ r:filter() { Ok(Expr::Or(Box::new(l?), Box::new(r?))) }
            / l:any_expr() _ "and" _ r:filter() { Ok(Expr::And(Box::new(l?), Box::new(r?))) }
            / any_expr()

        rule any_expr() -> Result<Expr, Error>
            = "(" _ e:filter() _ ")" { e }
            / l:value_expr() _ r:after_value_expr() { Ok(match r? {
                AfterValueExpr::Compare(op, r) => Expr::Compare(Box::new(l?), op, r),
                AfterValueExpr::In(r) => Expr::In(Box::new(l?), r),
                AfterValueExpr::End => l?,
            }) }

        rule after_value_expr() -> Result<AfterValueExpr, Error>
            = op:comparison_op() _ r:value_expr() { Ok(AfterValueExpr::Compare(op, Box::new(r?))) }
            / "in" _ "(" _ r:filter_list() _ ")" { Ok(AfterValueExpr::In(r?)) }
            / { Ok(AfterValueExpr::End) }

        rule value_expr() -> Result<Expr, Error>
            = function_call()
            / v:value() { Ok(Expr::Value(v?)) }
            / i:identifier() { Ok(Expr::Identifier(i)) }

        rule comparison_op() -> CompareOperator
            = "eq" { CompareOperator::Equal }
            / "ne" { CompareOperator::NotEqual }
            / "gt" { CompareOperator::GreaterThan }
            / "ge" { CompareOperator::GreaterOrEqual }
            / "lt" { CompareOperator::LessThan }
            / "le" { CompareOperator::LessOrEqual }

        rule function_call() -> Result<Expr, Error>
            = f:identifier() _ "(" _ l:filter_list() _ ")" { Ok(Expr::Function(f, l?)) }

        rule identifier() -> String
            = s:$(['a'..='z'|'A'..='Z'|'_']['a'..='z'|'A'..='Z'|'_'|'0'..='9']+) { s.to_string() }

        rule value() -> Result<Value, Error>
            = v:string_value() { Ok(v) }
            / datetime_value()
            / date_value()
            / time_value()
            / number_value()
            / v:bool_value() { Ok(v) }
            / v:null_value() { Ok(v) }

        rule bool_value() -> Value
            = ['t'|'T']['r'|'R']['u'|'U']['e'|'E'] { Value::Bool(true) }
            / ['f'|'F']['a'|'A']['l'|'L']['s'|'S']['e'|'E'] { Value::Bool(false) }

        rule number_value() -> Result<Value, Error>
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) { Ok(Value::Number(BigDecimal::from_str(n).map_err(|_| Error::ParsingNumber)?)) }

        rule time() -> Result<NaiveTime, Error>
            = t:$($(['0'..='9']*<2>) ":" $(['0'..='9']*<2>) ":" $(['0'..='9']*<2>)) { NaiveTime::parse_from_str(t, "%H:%M:%S").map_err(|_| Error::ParsingTime) }
            / t:$($(['0'..='9']*<2>) ":" $(['0'..='9']*<2>)) { NaiveTime::parse_from_str(t, "%H:%M").map_err(|_| Error::ParsingTime) }

        rule time_value() -> Result<Value, Error>
            = t:time() { Ok(Value::Time(t?)) }

        rule date() -> Result<NaiveDate, Error>
            = d:$($(['0'..='9']*<4>) "-" $(['0'..='9']*<2>) "-" $(['0'..='9']*<2>)) { NaiveDate::parse_from_str(d, "%Y-%m-%d").map_err(|_| Error::ParsingDate) }

        rule date_value() -> Result<Value, Error>
            = d:date() { Ok(Value::Date(d?)) }

        rule timezone_name() -> Result<chrono_tz::Tz, Error>
            = z:$(['a'..='z'|'A'..='Z'|'-'|'_'|'/'|'+']['a'..='z'|'A'..='Z'|'-'|'_'|'/'|'+'|'0'..='9']+) { z.parse::<chrono_tz::Tz>().map_err(|_| Error::ParsingTimeZoneNamed) }

        rule timezone_offset() -> Result<FixedOffset, Error>
            = "Z" { "+0000".parse().map_err(|_| Error::ParsingTimeZone) }
            / z:$($(['-'|'+']) $(['0'..='9']*<2>) ":"? $(['0'..='9']*<2>)) { z.parse().map_err(|_| Error::ParsingTimeZone) }
            / z:$($(['-'|'+']) $(['0'..='9']*<2>)) { format!("{z}00").parse().map_err(|_| Error::ParsingTimeZone) }

        rule datetime() -> Result<DateTime<Utc>, Error>
            = d:date() "T" t:time() z:timezone_offset() { Ok(d?.and_time(t?).and_local_timezone(z?).earliest().ok_or(Error::ParsingDateTime)?.to_utc()) }
            / d:date() "T" t:time() z:timezone_name() { Ok(d?.and_time(t?).and_local_timezone(z?).earliest().ok_or(Error::ParsingDateTime)?.to_utc()) }

        rule datetime_value() -> Result<Value, Error>
            = dt:datetime() { Ok(Value::DateTime(dt?)) }

        rule string_value() -> Value
            = "'" s:$([^'\'']*) "'" { Value::String(s.to_string()) }

        rule null_value() -> Value
            = ['n'|'N']['u'|'U']['l'|'L']['l'|'L'] { Value::Null }

        rule value_list() -> Result<Vec<Expr>, Error>
            = v:value_expr() ** ( _ "," _ ) { v.into_iter().collect() }

        rule filter_list() -> Result<Vec<Expr>, Error>
            = v:filter() ** ( _ "," _ ) { v.into_iter().collect() }

        rule _()
            = [' '|'\t'|'\n'|'\r']*
    }
}
