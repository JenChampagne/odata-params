use bigdecimal::BigDecimal;
use std::str::FromStr;

pub use odata_filter::parse_str;

#[derive(Debug)]
pub enum Expr {
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Compare(Box<Expr>, CompareOperator, Box<Expr>),
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
            / number_value()
            / bool_value()
            / null_value()

        rule bool_value() -> Value
            = ['t'|'T']['r'|'R']['u'|'U']['e'|'E'] { Value::Bool(true) }
            / ['f'|'F']['a'|'A']['l'|'L']['s'|'S']['e'|'E'] { Value::Bool(false) }

        rule number_value() -> Value
            = n:$(['0'..='9']+ ("." ['0'..='9']*)?) { Value::Number(BigDecimal::from_str(n).unwrap()) }

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
