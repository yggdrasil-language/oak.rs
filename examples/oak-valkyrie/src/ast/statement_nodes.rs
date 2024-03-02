//! Statement nodes for the Valkyrie language AST.
//!
//! This module defines block-level statements such as `let` bindings and
//! expression statements.

use super::{Attribute, Pattern, Span, TermExpression, TypeExpression};
use crate::ast::template_nodes::TemplateNode;

/// A `let` binding statement.
///
/// # V Language Example
/// ```v
/// micro demo() {
///     let count: i32 = 0
///     let (x, y) = (1, 2)
///     let Point { x: px, y: py } = point
/// }
/// ```
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
///
/// # V Language Example
/// ```v
/// micro demo() {
///     println("ready")
///     do_work()
/// }
/// ```
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

/// A statement inside a block.
///
/// # V Language Example
/// ```v
/// micro run() {
///     let total = 0
///     total = total + step()
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Statement {
    /// A let binding statement.
    Let(Let),
    /// An expression statement.
    Expression(Box<ExprStmt>),
    /// A TGrammar template node embedded in a block.
    Template(Box<TemplateNode>),
}
