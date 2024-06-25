use bigdecimal::BigDecimal;

pub use odata_filter::parse_str;

#[derive(Debug)]
pub enum Expr {
    Or(Box<Expr>, Box<Expr>),
    And(Box<Expr>, Box<Expr>),
    Identifier(String),
    Value(Value),
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
        use super::{Expr, Value};

        pub rule parse_str() -> Expr
            = filter()

        rule filter() -> Expr
            = l:any_expr() _ "or" _ r:filter() { Expr::Or(Box::new(l), Box::new(r)) }
            / l:any_expr() _ "and" _ r:filter() { Expr::And(Box::new(l), Box::new(r)) }
            / any_expr()

        rule any_expr() -> Expr
            = value_expr()

        rule value_expr() -> Expr
            = v:value() { Expr::Value(v) }
            / i:identifier() { Expr::Identifier(i) }

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

        rule _()
            = [' '|'\t'|'\n'|'\r']*
    }
}
