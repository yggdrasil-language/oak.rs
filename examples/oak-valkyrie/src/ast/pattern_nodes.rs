//! Pattern nodes for the Valkyrie language AST.
//!
//! This module defines patterns used by `let`, `match`, and extractor forms.

use super::{Identifier, NamePath, Span, TermExpression};

/// A `match` arm.
///
/// # V Language Example
/// ```v
/// match value {
///     0 => "zero"
///     n if n > 0 => "positive"
///     _ => "other"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MatchArm {
    /// The pattern to match against.
    pub pattern: Pattern,
    /// Optional guard expression.
    pub guard: Option<TermExpression>,
    /// The body expression of the arm.
    pub body: TermExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A pattern for matching or binding.
///
/// # V Language Example
/// ```v
/// let _ = value
/// let count = value
/// let Some { value: inner } = option
/// match token {
///     Identifier(name) => name
///     else => ""
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Pattern {
    /// A wildcard pattern that matches anything.
    Wildcard(Box<WildcardPattern>),
    /// A variable pattern that binds the matched value.
    Variable(Box<VariablePattern>),
    /// A literal pattern.
    Literal(Box<LiteralPattern>),
    /// A type pattern for matching types.
    Type(Box<TypePattern>),
    /// A class pattern for destructuring.
    Class(Box<ClassPattern>),
    /// An else pattern (catch-all).
    Else(Box<ElsePattern>),
    /// An array pattern with optional rest binding.
    Array(Box<ArrayPattern>),
    /// 备选模式：`case A | B: …`，任一子模式匹配即命中。
    Alternative(Box<AlternativePattern>),
}

/// 备选模式（`A | B | C`）。
///
/// 仅允许出现在 match arm 顶层；子模式按源码顺序保存，语义为“或”。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlternativePattern {
    /// 备选子模式，至少两个。
    pub patterns: Vec<Pattern>,
    /// 整个备选模式覆盖的源码区间。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A wildcard pattern that matches anything.
///
/// # V Language Example
/// ```v
/// match value {
///     _ => 0
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WildcardPattern {
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A variable pattern that binds the matched value.
///
/// # V Language Example
/// ```v
/// let count = 10
/// match value {
///     head => head
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariablePattern {
    /// The variable name.
    pub name: Identifier,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A literal pattern.
///
/// # V Language Example
/// ```v
/// match code {
///     200 => "ok"
///     404 => "missing"
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LiteralPattern {
    /// The literal value as written in source.
    pub value: String,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A type pattern for matching nominal shapes.
///
/// # V Language Example
/// ```v
/// match result {
///     Fine(value) => value
///     Fail(error) => panic(error)
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TypePattern {
    /// The type name path.
    pub name: NamePath,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A class pattern for destructuring.
///
/// # V Language Example
/// ```v
/// let Point { x, y } = p
/// let Point { x: a, y: b } = p
/// let Point { x, y: new_y } = p
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassPattern {
    /// The class name path.
    pub name: NamePath,
    /// The field patterns. `None` means shorthand syntax.
    pub fields: Vec<(Identifier, Option<Pattern>)>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An else pattern (catch-all) used in extractor forms.
///
/// # V Language Example
/// ```v
/// match bytes {
///     [head, ...tail] => head
///     else => 0
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ElsePattern {
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An array pattern with an optional rest binding.
///
/// # V Language Example
/// ```v
/// match bytes {
///     [head, ...tail] => head
///     [first, ..., last] => first
/// }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ArrayPattern {
    /// Patterns before the rest binding.
    pub prefix: Vec<Pattern>,
    /// The optional rest binding (`...rest`).
    pub rest: Option<Box<Pattern>>,
    /// Patterns after the rest binding.
    pub suffix: Vec<Pattern>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
