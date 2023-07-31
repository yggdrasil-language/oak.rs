use oak_core::{ElementType, UniversalElementRole};
use std::fmt;

/// Syntax node kinds for athena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AthenaElementType {
    /// Root.
    Root,
    /// Grouped expression.
    Expression,
    /// Function call `sin(x)`.
    Call,
    /// Argument list.
    Arguments,
    /// List literal `[a, b]`.
    List,
    /// Dict literal `{a: 1}`.
    Dict,
    /// Dict entry `k: v`.
    DictEntry,
    /// Symbol / variable.
    Symbol,
    /// Numeric / string literal.
    Literal,
    /// Binary expression.
    BinaryExpr,
    /// Prefix expression (unary minus).
    PrefixExpr,
    /// Error node.
    Error,
}

impl fmt::Display for AthenaElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ElementType for AthenaElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        UniversalElementRole::None
    }
}

impl From<crate::lexer::token_type::AthenaTokenType> for AthenaElementType {
    fn from(token: crate::lexer::token_type::AthenaTokenType) -> Self {
        use crate::lexer::token_type::AthenaTokenType as T;
        match token {
            T::Root => Self::Root,
            T::Identifier => Self::Symbol,
            T::Integer | T::Real | T::String => Self::Literal,
            _ => Self::Error,
        }
    }
}
