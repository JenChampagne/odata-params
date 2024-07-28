mod parse;
mod to_query_string;

use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use uuid::Uuid;

pub use parse::parse_str;
pub use to_query_string::{to_query_string, write_query_string};

/// Represents various errors that can occur during parsing.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    /// Error during general parsing.
    Parsing,

    /// Error parsing a uuid.
    ParsingUuid,

    /// Error parsing a number.
    ParsingNumber,

    /// Error parsing a date.
    ParsingDate,

    /// Error parsing a time.
    ParsingTime,

    /// Error parsing a datetime.
    ParsingDateTime,

    /// Error parsing a time zone offset.
    ParsingTimeZone,

    /// Error parsing a named time zone.
    ParsingTimeZoneNamed,

    /// Error parsing unicode code point escape sequence.
    ParsingUnicodeCodePoint,
}

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}

/// Represents the different types of expressions in the AST.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expr {
    /// Logical OR between two expressions.
    Or(Box<Expr>, Box<Expr>),

    /// Logical AND between two expressions.
    And(Box<Expr>, Box<Expr>),

    /// Comparison between two expressions.
    Compare(Box<Expr>, CompareOperator, Box<Expr>),

    /// In operator to check if a value is within a list of values.
    In(Box<Expr>, Vec<Expr>),

    /// Logical NOT to invert an expression.
    Not(Box<Expr>),

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
