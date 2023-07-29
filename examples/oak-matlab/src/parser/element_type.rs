//! Matlab element types (structural AST kinds for Pratt — not a token transmute).

use oak_core::{ElementType, UniversalElementRole};
use std::fmt;

/// Element types for Matlab expressions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MatlabElementType {
    /// File / expression root.
    Root,
    /// General grouped expression.
    Expression,
    /// Identifier / name.
    Symbol,
    /// Number or string literal.
    Literal,
    /// `f(…)` call.
    Call,
    /// Call argument list `(…)`.
    Arguments,
    /// `[a, b]` array / row.
    Array,
    /// Binary operator application.
    BinaryExpr,
    /// Prefix operator application.
    PrefixExpr,
    /// Postfix operator application.
    PostfixExpr,
    /// `if … end` statement.
    IfStmt,
    /// `while … end` statement.
    WhileStmt,
    /// `for … end` statement.
    ForStmt,
    /// `switch … end` statement.
    SwitchStmt,
    /// `try … catch … end` statement.
    TryStmt,
    /// Error node.
    Error,
}

impl fmt::Display for MatlabElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl ElementType for MatlabElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        UniversalElementRole::None
    }
}

impl From<crate::lexer::token_type::MatlabTokenType> for MatlabElementType {
    fn from(token: crate::lexer::token_type::MatlabTokenType) -> Self {
        use crate::lexer::token_type::MatlabTokenType as T;
        match token {
            T::Identifier => Self::Symbol,
            T::Number | T::String | T::Character => Self::Literal,
            _ => Self::Error,
        }
    }
}
