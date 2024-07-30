use bigdecimal::BigDecimal;
use odata_params::filters::{
    CompareOperator, Expr, FunctionsTypeMap, IdentifiersTypeMap, Type, ValidationError, Value,
};
use std::collections::HashMap;
use std::str::FromStr;

#[test]
fn test_literal_values() {
    let type_map = IdentifiersTypeMap::from(HashMap::new());
    let functions_map = FunctionsTypeMap::from(HashMap::new());

    assert_eq!(
        Expr::Value(Value::Bool(true)).validate(&type_map, &functions_map),
        Ok(Type::Boolean)
    );
    assert_eq!(
        Expr::Value(Value::Number(BigDecimal::from_str("42").unwrap()))
            .validate(&type_map, &functions_map),
        Ok(Type::Number)
    );
    assert_eq!(
        Expr::Value(Value::String("hello".to_string())).validate(&type_map, &functions_map),
        Ok(Type::String)
    );
}

#[test]
fn test_identifiers() {
    let mut id_map = HashMap::new();
    id_map.insert("abc".to_string(), Type::Number);
    let type_map = IdentifiersTypeMap::from(id_map);
    let functions_map = FunctionsTypeMap::from(HashMap::new());

    assert_eq!(
        Expr::Identifier("abc".to_string()).validate(&type_map, &functions_map),
        Ok(Type::Number)
    );
    assert_eq!(
        Expr::Identifier("unknown".to_string()).validate(&type_map, &functions_map),
        Err(ValidationError::UndefinedIdentifier {
            name: "unknown".to_string()
        })
    );
}

#[test]
fn test_comparisons() {
    let mut id_map = HashMap::new();
    id_map.insert("abc".to_string(), Type::Number);
    let type_map = IdentifiersTypeMap::from(id_map);
    let functions_map = FunctionsTypeMap::from(HashMap::new());

    let expr = Expr::Compare(
        Box::new(Expr::Identifier("abc".to_string())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::Number(
            BigDecimal::from_str("42").unwrap(),
        ))),
    );
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));

    let expr = Expr::Compare(
        Box::new(Expr::Identifier("abc".to_string())),
        CompareOperator::Equal,
        Box::new(Expr::Value(Value::String("42".to_string()))),
    );
    assert_eq!(
        expr.validate(&type_map, &functions_map),
        Err(ValidationError::ComparingIncompatibleTypes {
            lhs: Type::Number,
            rhs: Type::String,
        })
    );
}

#[test]
fn test_logical_operations() {
    let mut id_map = HashMap::new();
    id_map.insert("flag".to_string(), Type::Boolean);
    let type_map = IdentifiersTypeMap::from(id_map);
    let functions_map = FunctionsTypeMap::from(HashMap::new());

    let expr = Expr::And(
        Box::new(Expr::Identifier("flag".to_string())),
        Box::new(Expr::Value(Value::Bool(true))),
    );
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));

    let expr = Expr::Or(
        Box::new(Expr::Identifier("flag".to_string())),
        Box::new(Expr::Value(Value::Bool(false))),
    );
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));

    let expr = Expr::Not(Box::new(Expr::Identifier("flag".to_string())));
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));
}

#[test]
fn test_in_operator() {
    let mut id_map = HashMap::new();
    id_map.insert("id".to_string(), Type::Number);
    let type_map = IdentifiersTypeMap::from(id_map);
    let functions_map = FunctionsTypeMap::from(HashMap::new());

    let expr = Expr::In(
        Box::new(Expr::Identifier("id".to_string())),
        vec![
            Expr::Value(Value::Number(BigDecimal::from_str("1").unwrap())),
            Expr::Value(Value::Number(BigDecimal::from_str("2").unwrap())),
        ],
    );
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));

    let expr = Expr::In(
        Box::new(Expr::Identifier("id".to_string())),
        vec![
            Expr::Value(Value::Number(BigDecimal::from_str("1").unwrap())),
            Expr::Value(Value::String("2".to_string())),
        ],
    );
    assert_eq!(
        expr.validate(&type_map, &functions_map),
        Err(ValidationError::ComparingIncompatibleTypes {
            lhs: Type::Number,
            rhs: Type::String,
        })
    );
}

#[test]
fn test_function_call() {
    let mut id_map = HashMap::new();
    id_map.insert("arg1".to_string(), Type::Number);
    id_map.insert("arg2".to_string(), Type::String);
    let type_map = IdentifiersTypeMap::from(id_map);

    let mut func_map = HashMap::new();
    func_map.insert(
        "test_func".to_string(),
        (vec![Type::Number, Type::Number], None, Type::Boolean),
    );
    func_map.insert(
        "variadic_func".to_string(),
        (vec![Type::String], Some(Type::String), Type::Boolean),
    );
    let functions_map = FunctionsTypeMap::from(func_map);

    let expr = Expr::Function(
        "test_func".to_string(),
        vec![
            Expr::Identifier("arg1".to_string()),
            Expr::Value(Value::Number(BigDecimal::from_str("42").unwrap())),
        ],
    );
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));

    let expr = Expr::Function(
        "test_func".to_string(),
        vec![
            Expr::Identifier("arg1".to_string()),
            Expr::Value(Value::String("42".to_string())),
        ],
    );
    assert_eq!(
        expr.validate(&type_map, &functions_map),
        Err(ValidationError::IncorrectFunctionArgumentType {
            name: "test_func".to_string(),
            position: 2,
            expected: Type::Number,
            given: Type::String,
        })
    );

    let expr = Expr::Function(
        "variadic_func".to_string(),
        vec![
            Expr::Identifier("arg2".to_string()),
            Expr::Value(Value::String("foo".to_string())),
            Expr::Value(Value::String("bar".to_string())),
        ],
    );
    assert_eq!(expr.validate(&type_map, &functions_map), Ok(Type::Boolean));
}
