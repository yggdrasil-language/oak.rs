#![doc = include_str!("readme.md")]

use oak_core::{Language, LanguageCategory};

/// The Athena expression language (SXO simple-math / sm surface).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct AthenaLanguage {}

impl AthenaLanguage {
    /// Creates a new `AthenaLanguage`.
    pub fn new() -> Self {
        Self {}
    }
}

impl Language for AthenaLanguage {
    const NAME: &'static str = "athena";
    const CATEGORY: LanguageCategory = LanguageCategory::Programming;

    type TokenType = crate::lexer::token_type::AthenaTokenType;
    type ElementType = crate::parser::element_type::AthenaElementType;
    type TypedRoot = ();
}
