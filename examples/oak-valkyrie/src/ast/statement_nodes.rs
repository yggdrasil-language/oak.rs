use super::{Attribute, Pattern, Span, TermExpression, TypeExpression};

/// A let binding statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Let {
    /// The pattern to bind to.
    pub pattern: Pattern,
    /// The expression being bound.
    pub expr: TermExpression,
    /// Optional type annotation.
    pub ty: Option<TypeExpression>,
    /// Annotations applied to the statement.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An expression statement.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ExprStmt {
    /// The expression.
    pub expr: TermExpression,
    /// Whether the statement ends with a semicolon.
    pub semi: bool,
    /// Annotations applied to the statement.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A statement
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Statement {
    /// A let binding statement.
    Let(Let),
    /// An expression statement.
    ExprStmt(ExprStmt),
}
