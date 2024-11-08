//! Shared AST nodes such as attributes, literals, modifiers, and enum variants.

use super::{Block, FieldDeclaration, GenericParam, Identifier, Param, Pattern, Span, TermExpression, TypeExpression};

/// One attribute argument, optionally keyed (`name: "answer"`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeArgument {
    /// Optional named-argument key.
    pub key: Option<Identifier>,
    /// Structured argument expression.
    pub value: TermExpression,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// 声明修饰符；词法上是普通 `Identifier`，由 parser 在声明前位置建成此节点。
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Modifier {
    /// 修饰符名称（如 `abstract`、`sealed`、`lazy`，或任意标识符）。
    pub name: Identifier,
    /// 源码区间。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// An attribute
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attribute {
    /// The attribute name.
    pub name: Identifier,
    /// The attribute arguments.
    pub args: Vec<AttributeArgument>,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Quote character kind of a captured literal.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum QuoteKind {
    /// `'` capture.
    Single,
    /// `"` capture.
    Double,
}

impl QuoteKind {
    /// The quote character for this kind.
    pub fn character(self) -> char {
        match self {
            QuoteKind::Single => '\'',
            QuoteKind::Double => '"',
        }
    }
}

/// A quoted literal node: pure capture facts, no semantics.
///
/// The lexer captures raw text between equal-length delimiter runs; prefix,
/// quote kind and run length are syntax metadata. Character/text attribution,
/// escape decoding and template interpretation all belong to consumers after
/// capture; template structure comes from an explicit
/// [`crate::builder::ValkyrieBuilder::parse_quote_template`] request.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuoteLiteral {
    /// Adjacent identifier prefix (e.g., `r`, `b`); no semantic preset.
    pub prefix: Option<Identifier>,
    /// Quote character kind.
    pub quote: QuoteKind,
    /// Number of quotes in the delimiter run (1, 3, 4, ...; `""` / `''` empty captures use 2).
    pub quote_count: u8,
    /// Raw captured content between the delimiter runs, undecoded.
    pub content: String,
    /// Span of the raw content (excludes the prefix and both delimiter runs).
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub content_span: Span,
    /// The source code span of the whole literal (including the prefix).
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// A template segment produced by [`crate::builder::ValkyrieBuilder::parse_quote_template`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StringSegment {
    /// Raw text content: escapes are kept verbatim (`\{` / `\}` included), decoding belongs to consumers.
    Text(Box<TextSegment>),
    /// Interpolation carrying a real parsed expression tree with absolute source spans.
    Interpolation(Box<InterpolationSegment>),
}

/// Text content segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextSegment {
    /// The raw text content, undecoded.
    pub content: String,
    /// The source code span.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// Interpolation expression segment.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InterpolationSegment {
    /// The interpolation expression, parsed by the regular expression parser.
    pub expr: TermExpression,
    /// Whether this is a locale argument (with the ߷ locale marker).
    pub is_locale: bool,
    /// The source code span of the whole `{ ... }` region.
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
