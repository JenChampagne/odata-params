use odata_params::bigdecimal::BigDecimal;
use odata_params::filters::CompareOperator::*;
use odata_params::filters::{parse_str, Expr, Value};
use std::str::FromStr;

#[test]
fn or_grouping() {
    let filter = "name eq 'John' or isActive eq true";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Or(
            Expr::Compare(
                Expr::Identifier("name".to_owned()).into(),
                Equal,
                Expr::Value(Value::String("John".to_owned())).into()
            )
            .into(),
            Expr::Compare(
                Expr::Identifier("isActive".to_owned()).into(),
                Equal,
                Expr::Value(Value::Bool(true)).into()
            )
            .into()
        )
    );
}

#[test]
fn and_grouping() {
    let filter = "name eq 'John' and isActive eq true";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("name".to_owned()).into(),
                Equal,
                Expr::Value(Value::String("John".to_owned())).into()
            )
            .into(),
            Expr::Compare(
                Expr::Identifier("isActive".to_owned()).into(),
                Equal,
                Expr::Value(Value::Bool(true)).into()
            )
            .into()
        )
    );
}

#[test]
fn not_grouping() {
    let filter = "not (name eq 'John')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Not(
            Expr::Compare(
                Expr::Identifier("name".to_owned()).into(),
                Equal,
                Expr::Value(Value::String("John".to_owned())).into()
            )
            .into()
        )
    );
}

#[test]
fn complex_and_or_grouping() {
    let filter = "(name eq 'John' and isActive eq true) or age gt 30";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Or(
            Expr::And(
                Expr::Compare(
                    Expr::Identifier("name".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::String("John".to_owned())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("isActive".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::Bool(true)).into()
                )
                .into()
            )
            .into(),
            Expr::Compare(
                Expr::Identifier("age".to_owned()).into(),
                GreaterThan,
                Expr::Value(Value::Number(BigDecimal::from_str("30").unwrap())).into()
            )
            .into()
        )
    );
}

#[test]
fn nested_grouping() {
    let filter = "((name eq 'John' and isActive eq true) or (age gt 30 and age lt 50))";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Or(
            Expr::And(
                Expr::Compare(
                    Expr::Identifier("name".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::String("John".to_owned())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("isActive".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::Bool(true)).into()
                )
                .into()
            )
            .into(),
            Expr::And(
                Expr::Compare(
                    Expr::Identifier("age".to_owned()).into(),
                    GreaterThan,
                    Expr::Value(Value::Number(BigDecimal::from_str("30").unwrap())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("age".to_owned()).into(),
                    LessThan,
                    Expr::Value(Value::Number(BigDecimal::from_str("50").unwrap())).into()
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn function_call_endswith() {
    let filter = "endswith(name, 'Smith')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Function(
            "endswith".to_owned(),
            vec![
                Expr::Identifier("name".to_owned()),
                Expr::Value(Value::String("Smith".to_owned()))
            ]
        )
    );
}

#[test]
fn function_call_complex() {
    let filter = "concat(concat(city, ', '), country) eq 'Berlin, Germany'";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Function(
                "concat".to_owned(),
                vec![
                    Expr::Function(
                        "concat".to_owned(),
                        vec![
                            Expr::Identifier("city".to_owned()),
                            Expr::Value(Value::String(", ".to_owned()))
                        ]
                    ),
                    Expr::Identifier("country".to_owned())
                ]
            )
            .into(),
            Equal,
            Expr::Value(Value::String("Berlin, Germany".to_owned())).into()
        )
    );
}

#[test]
fn in_operator() {
    let filter = "name in ('John', 'Jane', 'Doe')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::In(
            Expr::Identifier("name".to_owned()).into(),
            vec![
                Expr::Value(Value::String("John".to_owned())),
                Expr::Value(Value::String("Jane".to_owned())),
                Expr::Value(Value::String("Doe".to_owned()))
            ]
        )
    );
}

#[test]
fn boolean_value() {
    let filter = "isActive eq false";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Identifier("isActive".to_owned()).into(),
            Equal,
            Expr::Value(Value::Bool(false)).into()
        )
    );
}

#[test]
fn number_value() {
    let filter = "price lt 99.99";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Identifier("price".to_owned()).into(),
            LessThan,
            Expr::Value(Value::Number(BigDecimal::from_str("99.99").unwrap())).into()
        )
    );
}

#[test]
fn nested_not() {
    let filter = "not (not (isActive eq false))";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Not(
            Expr::Not(
                Expr::Compare(
                    Expr::Identifier("isActive".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::Bool(false)).into()
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn complex_nested() {
    let filter = "((name eq 'John' and isActive eq true) or (age gt 30 and age lt 50)) and (city eq 'Berlin' or city eq 'Paris')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Or(
                Expr::And(
                    Expr::Compare(
                        Expr::Identifier("name".to_owned()).into(),
                        Equal,
                        Expr::Value(Value::String("John".to_owned())).into()
                    )
                    .into(),
                    Expr::Compare(
                        Expr::Identifier("isActive".to_owned()).into(),
                        Equal,
                        Expr::Value(Value::Bool(true)).into()
                    )
                    .into()
                )
                .into(),
                Expr::And(
                    Expr::Compare(
                        Expr::Identifier("age".to_owned()).into(),
                        GreaterThan,
                        Expr::Value(Value::Number(BigDecimal::from_str("30").unwrap())).into()
                    )
                    .into(),
                    Expr::Compare(
                        Expr::Identifier("age".to_owned()).into(),
                        LessThan,
                        Expr::Value(Value::Number(BigDecimal::from_str("50").unwrap())).into()
                    )
                    .into()
                )
                .into()
            )
            .into(),
            Expr::Or(
                Expr::Compare(
                    Expr::Identifier("city".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::String("Berlin".to_owned())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("city".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::String("Paris".to_owned())).into()
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn function_and_comparison() {
    let filter = "substring(name, 1, 3) eq 'Joh'";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Function(
                "substring".to_owned(),
                vec![
                    Expr::Identifier("name".to_owned()),
                    Expr::Value(Value::Number(BigDecimal::from_str("1").unwrap())),
                    Expr::Value(Value::Number(BigDecimal::from_str("3").unwrap()))
                ]
            )
            .into(),
            Equal,
            Expr::Value(Value::String("Joh".to_owned())).into()
        )
    );
}

#[test]
fn nested_function_calls() {
    let filter = "concat(substring(name, 1, 3), ' Doe') eq 'Joh Doe'";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Function(
                "concat".to_owned(),
                vec![
                    Expr::Function(
                        "substring".to_owned(),
                        vec![
                            Expr::Identifier("name".to_owned()),
                            Expr::Value(Value::Number(BigDecimal::from_str("1").unwrap())),
                            Expr::Value(Value::Number(BigDecimal::from_str("3").unwrap()))
                        ]
                    ),
                    Expr::Value(Value::String(" Doe".to_owned()))
                ]
            )
            .into(),
            Equal,
            Expr::Value(Value::String("Joh Doe".to_owned())).into()
        )
    );
}

#[test]
fn not_and_function() {
    let filter = "not endswith(name, 'Smith')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Not(
            Expr::Function(
                "endswith".to_owned(),
                vec![
                    Expr::Identifier("name".to_owned()),
                    Expr::Value(Value::String("Smith".to_owned()))
                ]
            )
            .into()
        )
    );
}

#[test]
fn mixed_operators() {
    let filter = "price gt 50.0 and (name eq 'John' or endswith(name, 'Doe'))";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("price".to_owned()).into(),
                GreaterThan,
                Expr::Value(Value::Number(BigDecimal::from_str("50.0").unwrap())).into()
            )
            .into(),
            Expr::Or(
                Expr::Compare(
                    Expr::Identifier("name".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::String("John".to_owned())).into()
                )
                .into(),
                Expr::Function(
                    "endswith".to_owned(),
                    vec![
                        Expr::Identifier("name".to_owned()),
                        Expr::Value(Value::String("Doe".to_owned()))
                    ]
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn not_in_operator() {
    let filter = "not name in ('John', 'Jane', 'Doe')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Not(
            Expr::In(
                Expr::Identifier("name".to_owned()).into(),
                vec![
                    Expr::Value(Value::String("John".to_owned())),
                    Expr::Value(Value::String("Jane".to_owned())),
                    Expr::Value(Value::String("Doe".to_owned()))
                ]
            )
            .into()
        )
    );
}

#[test]
fn nested_comparisons() {
    let filter =
        "((price gt 50.0 and price lt 100.0) or (discount eq 10.0 and isAvailable eq true))";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Or(
            Expr::And(
                Expr::Compare(
                    Expr::Identifier("price".to_owned()).into(),
                    GreaterThan,
                    Expr::Value(Value::Number(BigDecimal::from_str("50.0").unwrap())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("price".to_owned()).into(),
                    LessThan,
                    Expr::Value(Value::Number(BigDecimal::from_str("100.0").unwrap())).into()
                )
                .into()
            )
            .into(),
            Expr::And(
                Expr::Compare(
                    Expr::Identifier("discount".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::Number(BigDecimal::from_str("10.0").unwrap())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("isAvailable".to_owned()).into(),
                    Equal,
                    Expr::Value(Value::Bool(true)).into()
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn multiple_functions() {
    let filter = "startswith(name, 'J') and length(name) gt 3";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Function(
                "startswith".to_owned(),
                vec![
                    Expr::Identifier("name".to_owned()),
                    Expr::Value(Value::String("J".to_owned()))
                ]
            )
            .into(),
            Expr::Compare(
                Expr::Function(
                    "length".to_owned(),
                    vec![Expr::Identifier("name".to_owned())]
                )
                .into(),
                GreaterThan,
                Expr::Value(Value::Number(BigDecimal::from_str("3").unwrap())).into()
            )
            .into()
        )
    );
}

#[test]
fn boolean_function() {
    let filter = "isActive eq true and not contains(name, 'Admin')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("isActive".to_owned()).into(),
                Equal,
                Expr::Value(Value::Bool(true)).into()
            )
            .into(),
            Expr::Not(
                Expr::Function(
                    "contains".to_owned(),
                    vec![
                        Expr::Identifier("name".to_owned()),
                        Expr::Value(Value::String("Admin".to_owned()))
                    ]
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn nested_and_or_not() {
    let filter =
        "not ((price gt 50.0 or price lt 30.0) and not (discount eq 5.0 or discount eq 10.0))";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Not(
            Expr::And(
                Expr::Or(
                    Expr::Compare(
                        Expr::Identifier("price".to_owned()).into(),
                        GreaterThan,
                        Expr::Value(Value::Number(BigDecimal::from_str("50.0").unwrap())).into()
                    )
                    .into(),
                    Expr::Compare(
                        Expr::Identifier("price".to_owned()).into(),
                        LessThan,
                        Expr::Value(Value::Number(BigDecimal::from_str("30.0").unwrap())).into()
                    )
                    .into()
                )
                .into(),
                Expr::Not(
                    Expr::Or(
                        Expr::Compare(
                            Expr::Identifier("discount".to_owned()).into(),
                            Equal,
                            Expr::Value(Value::Number(BigDecimal::from_str("5.0").unwrap())).into()
                        )
                        .into(),
                        Expr::Compare(
                            Expr::Identifier("discount".to_owned()).into(),
                            Equal,
                            Expr::Value(Value::Number(BigDecimal::from_str("10.0").unwrap()))
                                .into()
                        )
                        .into()
                    )
                    .into()
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn date_comparison() {
    let filter = "birthdate eq 2024-06-24";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Identifier("birthdate".to_owned()).into(),
            Equal,
            Expr::Value(Value::Date("2024-06-24".parse().unwrap())).into()
        )
    );
}

#[test]
fn datetime_and_time() {
    let filter = "createdAt eq 2024-06-24T12:34:56Z and startTime lt 14:30:00";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("createdAt".to_owned()).into(),
                Equal,
                Expr::Value(Value::DateTime("2024-06-24T12:34:56Z".parse().unwrap())).into()
            )
            .into(),
            Expr::Compare(
                Expr::Identifier("startTime".to_owned()).into(),
                LessThan,
                Expr::Value(Value::Time("14:30:00".parse().unwrap())).into()
            )
            .into()
        )
    );
}

#[test]
fn decimal_comparison() {
    let filter = "rating eq 4.75";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Identifier("rating".to_owned()).into(),
            Equal,
            Expr::Value(Value::Number(BigDecimal::from_str("4.75").unwrap())).into()
        )
    );
}

#[test]
fn multiple_nested_functions() {
    let filter = "concat(concat(city, ', '), country) eq 'Berlin, Germany' and contains(description, 'sample')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Function(
                    "concat".to_owned(),
                    vec![
                        Expr::Function(
                            "concat".to_owned(),
                            vec![
                                Expr::Identifier("city".to_owned()),
                                Expr::Value(Value::String(", ".to_owned()))
                            ]
                        ),
                        Expr::Identifier("country".to_owned())
                    ]
                )
                .into(),
                Equal,
                Expr::Value(Value::String("Berlin, Germany".to_owned())).into()
            )
            .into(),
            Expr::Function(
                "contains".to_owned(),
                vec![
                    Expr::Identifier("description".to_owned()),
                    Expr::Value(Value::String("sample".to_owned()))
                ]
            )
            .into()
        )
    );
}
