use super::{Expr, FunctionsTypeMap, IdentifiersTypeMap, Type, ValidationError, Value};
use std::iter::repeat;

impl Expr {
    /// Validates if the types within the expression are correct and
    /// if the expression overall is a boolean type.
    ///
    /// A `Result` which is `Ok(true)` if the expression is a valid boolean
    /// expression, or an `Err` with a `ValidationError` if the types are not valid.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use odata_params::filters::{Expr, FunctionsTypeMap, IdentifiersTypeMap, Type};
    ///
    /// let mut id_map = HashMap::new();
    /// id_map.insert("value".to_string(), Type::Boolean);
    /// let identifiers = IdentifiersTypeMap::from(id_map);
    ///
    /// let functions = FunctionsTypeMap::from(HashMap::new());
    ///
    /// let expr = Expr::Identifier("value".to_string());
    ///
    /// assert_eq!(expr.are_types_valid(&identifiers, &functions), Ok(true));
    /// ```
    pub fn are_types_valid(
        &self,
        identifiers: &IdentifiersTypeMap,
        functions: &FunctionsTypeMap,
    ) -> Result<bool, ValidationError> {
        let overall_type = self.validate(identifiers, functions)?;

        Ok(overall_type == Type::Boolean)
    }

    /// Validates the types within the expression.
    ///
    /// A `Result` which is `Ok` with the type of the expression if the types
    /// are valid, or an `Err` with a `ValidationError` if the types are not valid.
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use odata_params::filters::{Expr, FunctionsTypeMap, IdentifiersTypeMap, Type};
    ///
    /// let mut id_map = HashMap::new();
    /// id_map.insert("value".to_string(), Type::Number);
    /// let identifiers = IdentifiersTypeMap::from(id_map);
    ///
    /// let mut func_map = HashMap::new();
    /// func_map.insert(
    ///     "sum".to_string(),
    ///     (vec![Type::Number], None, Type::Number),
    /// );
    /// let functions = FunctionsTypeMap::from(func_map);
    ///
    /// let expr = Expr::Function("sum".to_string(), vec![Expr::Identifier("value".to_string())]);
    ///
    /// assert_eq!(expr.validate(&identifiers, &functions), Ok(Type::Number));
    /// ```
    pub fn validate(
        &self,
        identifiers: &IdentifiersTypeMap,
        functions: &FunctionsTypeMap,
    ) -> Result<Type, ValidationError> {
        match self {
            Expr::Or(lhs, rhs) | Expr::And(lhs, rhs) => {
                let lhs_type = Self::validate(lhs, identifiers, functions)?;
                let rhs_type = Self::validate(rhs, identifiers, functions)?;

                if lhs_type == Type::Boolean && rhs_type == Type::Boolean {
                    Ok(Type::Boolean)
                } else {
                    Err(ValidationError::LogicalJoinRequiresBooleans {
                        lhs: lhs_type,
                        rhs: rhs_type,
                    })
                }
            }

            Expr::Not(inner) => {
                let inner_type = Self::validate(inner, identifiers, functions)?;

                if inner_type == Type::Boolean {
                    Ok(Type::Boolean)
                } else {
                    Err(ValidationError::LogicalNotRequiresBoolean { given: inner_type })
                }
            }

            Expr::Compare(lhs, _op, rhs) => {
                let lhs_type = Self::validate(lhs, identifiers, functions)?;
                let rhs_type = Self::validate(rhs, identifiers, functions)?;

                if lhs_type == rhs_type {
                    Ok(Type::Boolean)
                } else {
                    Err(ValidationError::ComparingIncompatibleTypes {
                        lhs: lhs_type,
                        rhs: rhs_type,
                    })
                }
            }

            Expr::In(lhs, values) => {
                let lhs_type = Self::validate(lhs, identifiers, functions)?;

                for value in values {
                    let value_type = Self::validate(value, identifiers, functions)?;

                    if lhs_type != value_type {
                        return Err(ValidationError::ComparingIncompatibleTypes {
                            lhs: lhs_type,
                            rhs: value_type,
                        });
                    }
                }

                Ok(Type::Boolean)
            }

            Expr::Function(function, args) => {
                let (types, variadic, ret) = functions.0.get(function).ok_or_else(|| {
                    ValidationError::UndefinedFunction {
                        name: function.to_owned(),
                    }
                })?;

                println!(":: {types:?}, {variadic:?}, {args:?}");

                if (variadic.is_none() && types.len() != args.len())
                    || (variadic.is_some() && types.len() > args.len())
                {
                    return Err(ValidationError::IncorrectFunctionArgumentsCount {
                        name: function.to_owned(),
                        is_variadic: variadic.is_some(),
                        expected: types.len(),
                        given: args.len(),
                    });
                }

                // It should be safe to setup an infinite chain of nulls when
                // `variadic` is not set since we should have already exited
                // early when `variadic` is None and `types` have a different
                // length than the given arguments.
                //
                // This is needed to have consistent types without needing to
                // collect eagerly. The `.zip` is what keeps the infinite
                // iterator fixed to the length of given arguments.
                let types = args.iter().zip(
                    types
                        .iter()
                        .copied()
                        .chain(repeat(variadic.unwrap_or(Type::Null))),
                );

                for (index, (arg, expected_type)) in types.enumerate() {
                    let arg_type = Self::validate(arg, identifiers, functions)?;

                    if arg_type != expected_type {
                        return Err(ValidationError::IncorrectFunctionArgumentType {
                            name: function.to_owned(),
                            position: index + 1,
                            expected: expected_type,
                            given: arg_type,
                        });
                    }
                }

                Ok(*ret)
            }

            Expr::Identifier(identifier) => {
                identifiers.0.get(identifier).copied().ok_or_else(|| {
                    ValidationError::UndefinedIdentifier {
                        name: identifier.to_owned(),
                    }
                })
            }

            Expr::Value(value) => Ok(match value {
                Value::Null => Type::Null,
                Value::Bool(_) => Type::Boolean,
                Value::Number(_) => Type::Number,
                Value::Uuid(_) => Type::Uuid,
                Value::DateTime(_) => Type::DateTime,
                Value::Date(_) => Type::Date,
                Value::Time(_) => Type::Time,
                Value::String(_) => Type::String,
            }),
        }
    }
}
