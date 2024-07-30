use super::{Expr, Value};
use chrono::SecondsFormat::Millis;
use std::fmt::{self, Write};

/// Converts an `Expr` AST to its corresponding OData `$filter` string representation.
///
/// A `Result` containing the resulting query string or a `fmt::Error`.
///
/// ```
/// use odata_params::bigdecimal::BigDecimal;
/// use odata_params::filters::{to_query_string, CompareOperator, Expr, Value};
///
/// let expr = Expr::Compare(
///     Box::new(Expr::Identifier("age".to_owned())),
///     CompareOperator::GreaterThan,
///     Box::new(Expr::Value(Value::Number(BigDecimal::from(30)))),
/// );
///
/// let query_string = to_query_string(&expr).expect("valid filter");
///
/// assert_eq!(query_string, "age gt 30");
/// ```
pub fn to_query_string(expr: &Expr) -> Result<String, fmt::Error> {
    let mut output = String::new();
    write_query_string(&mut output, expr)?;
    Ok(output)
}

/// Writes an `Expr` AST to a writer as its corresponding OData `$filter` string representation.
///
/// A `fmt::Result` indicating the success or failure of the write operation.
pub fn write_query_string<W: Write>(writer: &mut W, expr: &Expr) -> fmt::Result {
    write_string(writer, expr, false)
}

fn write_string<W: Write>(writer: &mut W, expr: &Expr, recursive_call: bool) -> fmt::Result {
    match expr {
        // Handle logical OR expressions.
        Expr::Or(lhs, rhs) => {
            // The recursive_call value is used here to not wrap the entire
            // output query in parentheses when the root `Expr` is an `Or`.
            if recursive_call {
                write!(writer, "(")?;
            }

            write_string(writer, lhs, true)?;
            write!(writer, " or ")?;

            if recursive_call {
                write_string(writer, rhs, true)?;
                write!(writer, ")")
            } else {
                write_string(writer, rhs, true)
            }
        }

        // Handle logical AND expressions.
        Expr::And(lhs, rhs) => {
            // The recursive_call value is used here to not wrap the entire
            // output query in parentheses when the root `Expr` is an `Or`.
            if recursive_call {
                write!(writer, "(")?;
            }

            write_string(writer, lhs, true)?;
            write!(writer, " and ")?;

            if recursive_call {
                write_string(writer, rhs, true)?;
                write!(writer, ")")
            } else {
                write_string(writer, rhs, true)
            }
        }

        // Handle comparison expressions.
        Expr::Compare(lhs, op, rhs) => {
            write_string(writer, lhs, true)?;
            write!(writer, " {op} ")?;
            write_string(writer, rhs, true)
        }

        // Handle IN expressions.
        Expr::In(lhs, values) => {
            write_string(writer, lhs, true)?;
            write!(writer, " in (")?;

            for (i, value) in values.iter().enumerate() {
                if i > 0 {
                    write!(writer, ", ")?;
                }

                write_string(writer, value, true)?;
            }

            write!(writer, ")")
        }

        // Handle logical NOT expressions.
        Expr::Not(expr) => {
            write!(writer, "not ")?;
            write_string(writer, expr, true)
        }

        // Handle function calls.
        Expr::Function(name, args) => {
            write!(writer, "{name}(")?;

            for (i, arg) in args.iter().enumerate() {
                if i > 0 {
                    write!(writer, ", ")?;
                }

                write_string(writer, arg, true)?;
            }

            write!(writer, ")")
        }

        // Handle identifiers.
        Expr::Identifier(name) => write!(writer, "{name}"),

        // Handle values.
        Expr::Value(value) => write_value(writer, value),
    }
}

/// Writes a `Value` to a writer.
///
/// A `fmt::Result` indicating the success or failure of the write operation.
fn write_value<W: Write>(writer: &mut W, value: &Value) -> fmt::Result {
    match value {
        // Handle null values.
        Value::Null => write!(writer, "null"),

        // Handle boolean values.
        Value::Bool(b) => write!(writer, "{b}"),

        // Handle numeric values.
        Value::Number(n) => write!(writer, "{n}"),

        // Handle UUID values.
        Value::Uuid(id) => write!(writer, "{id}"),

        // Handle datetime values.
        Value::DateTime(dt) => write!(writer, "{}", dt.to_rfc3339_opts(Millis, true)),

        // Handle date values.
        Value::Date(d) => write!(writer, "{d}"),

        // Handle time values.
        Value::Time(t) => write!(writer, "{t}"),

        // Handle string values, escaping single quotes.
        Value::String(s) => write!(writer, "'{}'", s.replace('\'', "''")),
    }
}
