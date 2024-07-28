use odata_params::bigdecimal::BigDecimal;
use odata_params::filters::CompareOperator::{self, *};
use odata_params::filters::{parse_str, Expr, Value};
use std::str::FromStr;

#[test]
fn null_value() {
    let filter = "CompanyName ne null";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Identifier("CompanyName".to_owned()).into(),
            NotEqual,
            Expr::Value(Value::Null).into()
        )
    );
}

#[test]
fn boolean_value() {
    let filter = "isActive eq false and not isBlocked eq true";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("isActive".to_owned()).into(),
                Equal,
                Expr::Value(Value::Bool(false)).into()
            )
            .into(),
            Expr::Not(
                Expr::Compare(
                    Expr::Identifier("isBlocked".to_owned()).into(),
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
fn uuid_value() {
    let filter = [
        "AuthorId eq d1fdd9d1-8c73-4eb9-a341-3505d4efad78",
        "and PackageId ne C0BD12F1-9CAD-4081-977A-04B5AF7EDA0E",
    ]
    .join(" ");
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("AuthorId".to_owned()).into(),
                Equal,
                Expr::Value(Value::Uuid(uuid::uuid!(
                    "d1fdd9d1-8c73-4eb9-a341-3505d4efad78"
                )))
                .into()
            )
            .into(),
            Expr::Compare(
                Expr::Identifier("PackageId".to_owned()).into(),
                NotEqual,
                Expr::Value(Value::Uuid(uuid::uuid!(
                    "c0bd12f1-9cad-4081-977a-04b5af7eda0e"
                )))
                .into()
            )
            .into()
        )
    );
}

#[test]
fn number_value() {
    let filter = "price lt 99.99 and code in (11, 27, 42)";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Compare(
                Expr::Identifier("price".to_owned()).into(),
                LessThan,
                Expr::Value(Value::Number(BigDecimal::from_str("99.99").unwrap())).into()
            )
            .into(),
            Expr::In(
                Expr::Identifier("code".to_owned()).into(),
                vec![
                    Expr::Value(Value::Number(11.into())),
                    Expr::Value(Value::Number(27.into())),
                    Expr::Value(Value::Number(42.into())),
                ]
            )
            .into()
        )
    );
}

#[test]
fn date_value() {
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
fn time_value() {
    let filter = "(startTime lt 14:30:00 or pauseTime ge 13:00) and endTime le 8:00:00.001002003";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::And(
            Expr::Or(
                Expr::Compare(
                    Expr::Identifier("startTime".to_owned()).into(),
                    CompareOperator::LessThan,
                    Expr::Value(Value::Time("14:30:00".parse().unwrap())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("pauseTime".to_owned()).into(),
                    CompareOperator::GreaterOrEqual,
                    Expr::Value(Value::Time("13:00:00".parse().unwrap())).into()
                )
                .into()
            )
            .into(),
            Expr::Compare(
                Expr::Identifier("endTime".to_owned()).into(),
                CompareOperator::LessOrEqual,
                Expr::Value(Value::Time("08:00:00.001002003".parse().unwrap())).into()
            )
            .into()
        )
    );
}

#[test]
fn datetime_value() {
    let filter = [
        "   AT eq 2024-06-24T12:34:56Z",
        "or AT gt 2024-06-24T12:34:56+02",
        "or AT lt 2024-06-24T12:34:56EST",
    ]
    .join(" ");
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Or(
            Expr::Compare(
                Expr::Identifier("AT".to_owned()).into(),
                CompareOperator::Equal,
                Expr::Value(Value::DateTime("2024-06-24T12:34:56Z".parse().unwrap())).into()
            )
            .into(),
            Expr::Or(
                Expr::Compare(
                    Expr::Identifier("AT".to_owned()).into(),
                    CompareOperator::GreaterThan,
                    Expr::Value(Value::DateTime("2024-06-24T10:34:56Z".parse().unwrap())).into()
                )
                .into(),
                Expr::Compare(
                    Expr::Identifier("AT".to_owned()).into(),
                    CompareOperator::LessThan,
                    Expr::Value(Value::DateTime("2024-06-24T17:34:56Z".parse().unwrap())).into()
                )
                .into()
            )
            .into()
        )
    );
}

#[test]
fn string_value() {
    let filter = "Name in ('Ada', 'Joey')";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::In(
            Expr::Identifier("Name".to_owned()).into(),
            vec![
                Expr::Value(Value::String("Ada".to_owned())),
                Expr::Value(Value::String("Joey".to_owned())),
            ],
        )
    );
}

#[test]
fn escaped_string_comparison() {
    let filter = r"name eq '\u03A9 S\'mores'";
    let result = parse_str(filter).expect("valid filter tree");

    assert_eq!(
        result,
        Expr::Compare(
            Expr::Identifier("name".to_owned()).into(),
            Equal,
            Expr::Value(Value::String(String::from("Ω S'mores"))).into()
        )
    );
}
