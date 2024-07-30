mod parse;
mod to_query_string;
mod validate;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use std::collections::HashMap;
use thiserror::Error;
use uuid::Uuid;

pub use parse::parse_str;
pub use to_query_string::{to_query_string, write_query_string};

/// This alias is to make the rename to ParseError a non-breaking change.
/// You should prefer using ParseError.
#[deprecated = "Use ParseError instead."]
pub use ParseError as Error;

/// Represents various errors that can occur during parsing.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ParseError {
    /// Error during general parsing.
    #[error("Error during general parsing.")]
    Parsing,

    /// Error parsing a UUID.
    #[error("Error parsing a UUID.")]
    ParsingUuid,

    /// Error parsing a number.
    #[error("Error parsing a number.")]
    ParsingNumber,

    /// Error parsing a date.
    #[error("Error parsing a date.")]
    ParsingDate,

    /// Error parsing a time.
    #[error("Error parsing a time.")]
    ParsingTime,

    /// Error parsing a datetime.
    #[error("Error parsing a date and time.")]
    ParsingDateTime,

    /// Error parsing a time zone offset.
    #[error("Error parsing a time zone offset.")]
    ParsingTimeZone,

    /// Error parsing a named time zone.
    #[error("Error parsing a named time zone.")]
    ParsingTimeZoneNamed,

    /// Error parsing a Unicode code point escape sequence.
    #[error("Error parsing a Unicode code point escape sequence.")]
    ParsingUnicodeCodePoint,
}

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum ValidationError {
    /// Logical join (AND/OR) requires both sides to be booleans.
    #[error("Logical join requires boolean operands: lhs = {lhs:?}, rhs = {rhs:?}.")]
    LogicalJoinRequiresBooleans { lhs: Type, rhs: Type },

    /// Logical NOT requires a boolean operand.
    #[error("Logical NOT requires a boolean operand but got {given:?}.")]
    LogicalNotRequiresBoolean { given: Type },

    /// Comparison between incompatible types.
    #[error("Comparing incompatible types: lhs = {lhs:?}, rhs = {rhs:?}.")]
    ComparingIncompatibleTypes { lhs: Type, rhs: Type },

    /// Undefined identifier.
    #[error("Undefined identifier '{name}'.")]
    UndefinedIdentifier { name: String },

    /// Undefined function.
    #[error("Undefined function '{name}'.")]
    UndefinedFunction { name: String },

    /// Incorrect number of function arguments.
    #[error(
        "Function '{name}' expected {expected}{} arguments but got {given}.",
        if *is_variadic { " or more" } else { "" }
    )]
    IncorrectFunctionArgumentsCount {
        name: String,
        is_variadic: bool,
        expected: usize,
        given: usize,
    },

    /// Incorrect type for a function argument.
    #[error("Function '{name}' argument {position} expected type {expected:?} but got {given:?}.")]
    IncorrectFunctionArgumentType {
        name: String,
        position: usize,
        expected: Type,
        given: Type,
    },
}

/// Represents the different types of expressions in the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// Logical OR between two expressions.
    Or(Box<Expr>, Box<Expr>),

    /// Logical AND between two expressions.
    And(Box<Expr>, Box<Expr>),

    /// Logical NOT to invert an expression.
    Not(Box<Expr>),

    /// Comparison between two expressions.
    Compare(Box<Expr>, CompareOperator, Box<Expr>),

    /// In operator to check if a value is within a list of values.
    In(Box<Expr>, Vec<Expr>),

    /// Function call with a name and a list of arguments.
    Function(String, Vec<Expr>),

    /// An identifier.
    Identifier(String),

    /// A constant value.
    Value(Value),
}

/// Represents the various comparison operators.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompareOperator {
    /// Equal to.
    Equal,

    /// Not equal to.
    NotEqual,

    /// Greater than.
    GreaterThan,

    /// Greater than or equal to.
    GreaterOrEqual,

    /// Less than.
    LessThan,

    /// Less than or equal to.
    LessOrEqual,
}

/// Converts a `CompareOperator` to its string representation.
impl std::fmt::Display for CompareOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompareOperator::Equal => write!(f, "eq"),
            CompareOperator::NotEqual => write!(f, "ne"),
            CompareOperator::GreaterThan => write!(f, "gt"),
            CompareOperator::GreaterOrEqual => write!(f, "ge"),
            CompareOperator::LessThan => write!(f, "lt"),
            CompareOperator::LessOrEqual => write!(f, "le"),
        }
    }
}

/// Represents the various value types.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    /// Null value.
    Null,

    /// Boolean value.
    Bool(bool),

    /// Numeric value.
    Number(BigDecimal),

    /// Unique ID sometimes referred to as GUIDs.
    Uuid(Uuid),

    /// Date and time with time zone value.
    DateTime(DateTime<Utc>),

    /// Date value.
    Date(NaiveDate),

    /// Time value.
    Time(NaiveTime),

    /// String value.
    String(String),
}

#[derive(Copy, Clone, Debug, Eq)]
pub enum Type {
    Null,
    Boolean,
    Number,
    Uuid,
    DateTime,
    Date,
    Time,
    String,
}

impl PartialEq for Type {
    fn eq(&self, other: &Self) -> bool {
        use core::mem::discriminant as variant;

        variant(self) == variant(other)
            || variant(other) == variant(&Type::Null)
            || variant(self) == variant(&Type::Null)
    }
}

/// Represents a map of identifiers to their corresponding types.
///
/// ```
/// use std::collections::HashMap;
/// use odata_params::filters::{IdentifiersTypeMap, Type};
///
/// let mut map = HashMap::new();
/// map.insert("x".to_string(), Type::Number);
///
/// let identifiers_map: IdentifiersTypeMap = map.into();
/// ```
pub struct IdentifiersTypeMap(HashMap<String, Type>);

/// Represents a map of functions to their corresponding argument types, optional variadic argument type, and return type.
///
/// ```
/// use std::collections::HashMap;
/// use odata_params::filters::{FunctionsTypeMap, Type};
///
/// let mut map = HashMap::new();
///
/// // Support a `sum` function that takes one `Number` kind as input.
/// // Returns a `Number`.
/// map.insert(
///     "sum".to_string(),
///     (vec![Type::Number], None, Type::Number)
/// );
///
/// // Support an `any` function that takes at least one `Boolean` kind
/// // and any number of extra `Boolean` kind of inputs. Returns a `Boolean`.
/// //
/// // ex: any(IsActive, Name eq 'Jenny', has_role(Admin))
/// //     Returns `true` if any of the three conditions return `true`.
/// map.insert(
///     "any".to_string(),
///     (vec![Type::Boolean], Some(Type::Boolean), Type::Boolean)
/// );
///
/// let functions_map: FunctionsTypeMap = map.into();
/// ```
pub struct FunctionsTypeMap(HashMap<String, (Vec<Type>, Option<Type>, Type)>);

impl From<HashMap<String, Type>> for IdentifiersTypeMap {
    fn from(map: HashMap<String, Type>) -> Self {
        Self(map)
    }
}

impl From<HashMap<String, (Vec<Type>, Option<Type>, Type)>> for FunctionsTypeMap {
    fn from(map: HashMap<String, (Vec<Type>, Option<Type>, Type)>) -> Self {
        Self(map)
    }
}
