#![doc = include_str!("readme.md")]
use crate::ast::GlslRoot;
use oak_core::{Language, LanguageCategory};

/// GLSL language definition.
#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GlslLanguage {}

impl GlslLanguage {
    /// Creates a new GLSL language definition.
    pub fn new() -> Self {
        Self {}
    }
}

impl Language for GlslLanguage {
    const NAME: &'static str = "glsl";
    const CATEGORY: LanguageCategory = LanguageCategory::Programming;

    type TokenType = crate::lexer::token_type::GlslTokenType;
    type ElementType = crate::parser::element_type::GlslElementType;
    type TypedRoot = GlslRoot;
}
