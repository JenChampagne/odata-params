use odata_params::bigdecimal::BigDecimal;
use odata_params::chrono::{NaiveDate, NaiveTime, TimeZone, Utc};
use odata_params::filters::{to_query_string, CompareOperator, Expr, Value};

#[test]
fn or_grouping() {
    let expr = Expr::Or(
        Box::new(Expr::Compare(
            Box::new(Expr::Identifier("name".to_owned())),
            CompareOperator::Equal,
            Box::new(Expr::Value(Value::String("John".to_owned()))),
        )),
        Box::new(Expr::Compare(
            Box::new(Expr::Identifier("age".to_owned())),
            CompareOperator::LessThan,
            Box::new(Expr::Value(Value::Number(BigDecimal::from(25)))),
        )),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "name eq 'John' or age lt 25");
}

#[test]
fn and_grouping() {
    let expr = Expr::And(
        Box::new(Expr::Compare(
            Box::new(Expr::Identifier("age".to_owned())),
            CompareOperator::GreaterThan,
            Box::new(Expr::Value(Value::Number(BigDecimal::from(30)))),
        )),
        Box::new(Expr::Compare(
            Box::new(Expr::Identifier("isActive".to_owned())),
            CompareOperator::Equal,
            Box::new(Expr::Value(Value::Bool(true))),
        )),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "age gt 30 and isActive eq true");
}

#[test]
fn not_grouping() {
    let expr = Expr::Not(Box::new(Expr::Compare(
        Box::new(Expr::Identifier("name".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::String("John".to_owned()))),
    )));

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "not name eq 'John'");
}

#[test]
fn complex_and_or_grouping() {
    let expr = Expr::Or(
        Box::new(Expr::And(
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("name".to_owned())),
                CompareOperator::Equal,
                Box::new(Expr::Value(Value::String("John".to_owned()))),
            )),
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("isActive".to_owned())),
                CompareOperator::Equal,
                Box::new(Expr::Value(Value::Bool(true))),
            )),
        )),
        Box::new(Expr::And(
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("age".to_owned())),
                CompareOperator::GreaterThan,
                Box::new(Expr::Value(Value::Number(BigDecimal::from(30)))),
            )),
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("age".to_owned())),
                CompareOperator::LessThan,
                Box::new(Expr::Value(Value::Number(BigDecimal::from(50)))),
            )),
        )),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(
        result,
        "(name eq 'John' and isActive eq true) or (age gt 30 and age lt 50)"
    );
}

#[test]
fn simple_comparison() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("age".to_owned())),
        CompareOperator::GreaterThan,
        Box::new(Expr::Value(Value::Number(BigDecimal::from(30)))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "age gt 30");
}

#[test]
fn nested_grouping() {
    let expr = Expr::And(
        Box::new(Expr::Or(
            Box::new(Expr::Function(
                "startswith".to_owned(),
                vec![
                    Expr::Identifier("name".to_owned()),
                    Expr::Value(Value::String("J".to_owned())),
                ],
            )),
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("age".to_owned())),
                CompareOperator::LessThan,
                Box::new(Expr::Value(Value::Number(BigDecimal::from(25)))),
            )),
        )),
        Box::new(Expr::Compare(
            Box::new(Expr::Identifier("isActive".to_owned())),
            CompareOperator::Equal,
            Box::new(Expr::Value(Value::Bool(true))),
        )),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(
        result,
        "(startswith(name, 'J') or age lt 25) and isActive eq true"
    );
}

#[test]
fn function_call_startswith() {
    let expr = Expr::Function(
        "startswith".to_owned(),
        vec![
            Expr::Identifier("name".to_owned()),
            Expr::Value(Value::String("J".to_owned())),
        ],
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "startswith(name, 'J')");
}

#[test]
fn multiple_functions() {
    let expr = Expr::Compare(
        Box::new(Expr::Function(
            "concat".to_owned(),
            vec![
                Expr::Function(
                    "concat".to_owned(),
                    vec![
                        Expr::Identifier("City".to_owned()),
                        Expr::Value(Value::String(", ".to_owned())),
                    ],
                ),
                Expr::Identifier("Country".to_owned()),
            ],
        )),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::String("Berlin, Germany".to_owned()))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(
        result,
        "concat(concat(City, ', '), Country) eq 'Berlin, Germany'"
    );
}

#[test]
fn function_with_and_or() {
    let expr = Expr::And(
        Box::new(Expr::Function(
            "startswith".to_owned(),
            vec![
                Expr::Identifier("name".to_owned()),
                Expr::Value(Value::String("J".to_owned())),
            ],
        )),
        Box::new(Expr::Or(
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("age".to_owned())),
                CompareOperator::LessThan,
                Box::new(Expr::Value(Value::Number(BigDecimal::from(25)))),
            )),
            Box::new(Expr::Compare(
                Box::new(Expr::Identifier("isActive".to_owned())),
                CompareOperator::Equal,
                Box::new(Expr::Value(Value::Bool(true))),
            )),
        )),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(
        result,
        "startswith(name, 'J') and (age lt 25 or isActive eq true)"
    );
}

#[test]
fn in_operator() {
    let expr = Expr::In(
        Box::new(Expr::Identifier("name".to_owned())),
        vec![
            Expr::Value(Value::String("Milk".to_owned())),
            Expr::Value(Value::String("Cheese".to_owned())),
        ],
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "name in ('Milk', 'Cheese')");
}

#[test]
fn boolean() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("isValid".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::Bool(true))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "isValid eq true");
}

#[test]
fn number() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("age".to_owned())),
        CompareOperator::NotEqual,
        Box::new(Expr::Value(Value::Number(BigDecimal::from(42)))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "age ne 42");
}

#[test]
fn nested_not() {
    let expr = Expr::Not(Box::new(Expr::Not(Box::new(Expr::Compare(
        Box::new(Expr::Identifier("isActive".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::Bool(true))),
    )))));

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "not not isActive eq true");
}

#[test]
fn datetime() {
    let datetime = Utc.with_ymd_and_hms(2023, 6, 25, 13, 0, 0).unwrap();
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("created".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::DateTime(datetime))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "created eq 2023-06-25T13:00:00.000Z");
}

#[test]
fn date() {
    let date = NaiveDate::from_ymd_opt(2023, 6, 25).expect("valid date");
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("birthdate".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::Date(date))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "birthdate eq 2023-06-25");
}

#[test]
fn time() {
    let time = NaiveTime::from_hms_opt(13, 0, 0).expect("valid time");
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("startTime".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::Time(time))),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "startTime eq 13:00:00");
}

#[test]
fn null_value() {
    let expr = Expr::Compare(
        Box::new(Expr::Identifier("description".to_owned())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::Null)),
    );

    let result = to_query_string(&expr).expect("valid filter");
    assert_eq!(result, "description eq null");
}
