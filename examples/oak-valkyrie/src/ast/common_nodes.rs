use super::{Block, FieldDeclaration, GenericParam, Identifier, Param, Pattern, Span, TermExpression, TypeExpression};

/// An attribute
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attribute {
    /// The attribute name.
    pub name: Identifier,
    /// The attribute arguments.
    pub args: Vec<TermExpression>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A string literal node.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StringLiteral {
    /// DSL prefix (e.g., `s`, `f`, `r`, `sql`).
    pub prefix: Option<Identifier>,
    /// Number of quotes (1, 2, 3, 4, ...).
    pub quote_count: u8,
    /// String segments.
    pub segments: Vec<StringSegment>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A string segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringSegment {
    /// Text content.
    Text(Box<TextSegment>),
    /// Interpolation expression.
    Interpolation(Box<InterpolationSegment>),
}

/// Text content segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextSegment {
    /// The text content.
    pub content: String,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Interpolation expression segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterpolationSegment {
    /// The interpolation expression.
    pub expr: TermExpression,
    /// Whether this is a locale argument (with the ߷ locale marker).
    pub is_locale: bool,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An enum variant
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EnumVariant {
    /// The variant name.
    pub name: Identifier,
    /// The variant fields.
    pub fields: Vec<FieldDeclaration>,
    /// Annotations applied to the variant.
    pub annotations: Vec<Attribute>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
    /// Optional value expression for flags (e.g., `READ = 1` or `ALL = READ | WRITE`).
    pub value: Option<TermExpression>,
}

/// A variant case
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VariantCase {
    /// The pattern for this case.
    pub pattern: Pattern,
    /// The body expression for this case.
    pub body: TermExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}
