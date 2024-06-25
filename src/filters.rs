use bigdecimal::BigDecimal;
use chrono::TimeZone;
use chrono::{DateTime, FixedOffset, NaiveDate, NaiveTime, Utc};
use std::str::FromStr;

pub use odata_filter::parse_str;

#[derive(Debug)]
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

#[derive(Debug)]
pub enum CompareOperator {
    Equal,
    NotEqual,
    GreaterThan,
    GreaterOrEqual,
    LessThan,
    LessOrEqual,
}

#[derive(Debug)]
pub enum Value {
    Null,
    Bool(bool),
    Number(BigDecimal),
    DateTime(DateTime<Utc>),
    Date(NaiveDate),
    Time(NaiveTime),
    String(String),
}

peg::parser! {
    grammar odata_filter() for str {
        use super::{Expr, CompareOperator, Value};

        pub rule parse_str() -> Expr
            = filter()

        rule filter() -> Expr
            = "not" _ e:filter() { Expr::Not(Box::new(e)) }
            / l:any_expr() _ "or" _ r:filter() { Expr::Or(Box::new(l), Box::new(r)) }
            / l:any_expr() _ "and" _ r:filter() { Expr::And(Box::new(l), Box::new(r)) }
            / any_expr()

        rule any_expr() -> Expr
            = comparison_expr()
            / value_expr()
            / "(" _ e:filter() _ ")" { e }

        rule value_expr() -> Expr
            = function_call()
            / v:value() { Expr::Value(v) }
            / i:identifier() { Expr::Identifier(i) }

        rule comparison_expr() -> Expr
            = l:value_expr() _ op:comparison_op() _ r:value_expr() { Expr::Compare(Box::new(l), op, Box::new(r)) }
            / l:value_expr() _ "in" _ "(" _ r:filter_list() _ ")" { Expr::In(Box::new(l), r) }

        rule comparison_op() -> CompareOperator
            = "eq" { CompareOperator::Equal }
            / "ne" { CompareOperator::NotEqual }
            / "gt" { CompareOperator::GreaterThan }
            / "ge" { CompareOperator::GreaterOrEqual }
            / "lt" { CompareOperator::LessThan }
            / "le" { CompareOperator::LessOrEqual }

        rule function_call() -> Expr
            = fname:identifier() _ "(" _ args:filter_list() _ ")" { Expr::Function(fname, args) }

        rule identifier() -> String
            = s:$(['a'..='z'|'A'..='Z'|'_']['a'..='z'|'A'..='Z'|'_'|'0'..='9']+) { s.to_string() }

        rule value() -> Value
            = string_value()
            / datetime_value()
            / date_value()
            / time_value()
            / number_value()
            / bool_value()
            / null_value()

        rule bool_value() -> Value
            = ['t'|'T']['r'|'R']['u'|'U']['e'|'E'] { Value::Bool(true) }
            / ['f'|'F']['a'|'A']['l'|'L']['s'|'S']['e'|'E'] { Value::Bool(false) }

        rule number_value() -> Value
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) { Value::Number(BigDecimal::from_str(n).unwrap()) }

        rule time() -> NaiveTime
            = t:$($(['0'..='9']*<2>) ":" $(['0'..='9']*<2>) ":" $(['0'..='9']*<2>)) { NaiveTime::parse_from_str(t, "%H:%M:%S").unwrap() }
            / t:$($(['0'..='9']*<2>) ":" $(['0'..='9']*<2>)) { NaiveTime::parse_from_str(t, "%H:%M").unwrap() }

        rule time_value() -> Value
            = t:time() { Value::Time(t) }

        rule date() -> NaiveDate
            = d:$($(['0'..='9']*<4>) "-" $(['0'..='9']*<2>) "-" $(['0'..='9']*<2>)) { NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap() }

        rule date_value() -> Value
            = d:date() { Value::Date(d) }

        rule timezone_name() -> chrono_tz::Tz
            = z:$(['a'..='z'|'A'..='Z'|'-'|'_'|'/'|'+']['a'..='z'|'A'..='Z'|'-'|'_'|'/'|'+'|'0'..='9']+) { z.parse::<chrono_tz::Tz>().unwrap() }

        rule timezone_offset() -> FixedOffset
            = "Z" { "+0000".parse().unwrap() }
            / z:$($(['-'|'+']) $(['0'..='9']*<2>) ":"? $(['0'..='9']*<2>)) { z.parse().unwrap() }
            / z:$($(['-'|'+']) $(['0'..='9']*<2>)) { format!("{z}00").parse().unwrap() }

        rule datetime() -> DateTime<Utc>
            = d:date() "T" t:time() z:timezone_offset() { d.and_time(t).and_local_timezone(z).unwrap().to_utc() }
            / d:date() "T" t:time() z:timezone_name() { d.and_time(t).and_local_timezone(z).unwrap().to_utc() }

        rule datetime_value() -> Value
            = dt:datetime() { Value::DateTime(dt) }

        rule string_value() -> Value
            = "'" s:$([^'\'']*) "'" { Value::String(s.to_string()) }

        rule null_value() -> Value
            = ['n'|'N']['u'|'U']['l'|'L']['l'|'L'] { Value::Null }

        rule value_list() -> Vec<Expr>
            = v:value_expr() ** ( _ "," _ ) { v }

        rule filter_list() -> Vec<Expr>
            = v:filter() ** ( _ "," _ ) { v }

        rule _()
            = [' '|'\t'|'\n'|'\r']*
    }
}
