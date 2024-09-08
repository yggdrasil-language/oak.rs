#![doc = include_str!("readme.md")]
use oak_core::{Language, LanguageCategory};

/// DOT language configuration (Graphviz).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotLanguage {
    /// Whether to enable strict mode.
    pub strict_mode: bool,
    /// Whether to allow directed graphs.
    pub allow_digraph: bool,
}

impl DotLanguage {
    /// Creates a new DOT language configuration.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for DotLanguage {
    fn default() -> Self {
        Self { strict_mode: false, allow_digraph: true }
    }
}

impl Language for DotLanguage {
    const NAME: &'static str = "dot";
    const CATEGORY: LanguageCategory = LanguageCategory::Programming;

    type TokenType = crate::lexer::token_type::DotTokenType;
    type ElementType = crate::parser::element_type::DotElementType;
    type TypedRoot = crate::ast::DotRoot;
}
