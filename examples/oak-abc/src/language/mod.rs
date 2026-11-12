//! Language definition for ABC notation.

use oak_core::language::{Language, LanguageCategory};

use crate::{
    ast::AbcRoot, lexer::token_type::AbcTokenType, parser::element_type::AbcElementType,
};

/// Language definition for ABC notation.
pub struct AbcLanguage;

impl Language for AbcLanguage {
    const NAME: &'static str = "abc";
    const CATEGORY: LanguageCategory = LanguageCategory::Dsl;

    type TokenType = AbcTokenType;
    type ElementType = AbcElementType;
    type TypedRoot = AbcRoot;
}

impl Default for AbcLanguage {
    fn default() -> Self {
        Self
    }
}

impl AbcLanguage {
    /// Returns a new parser for this language.
    pub fn parser(&self) -> crate::parser::AbcParser {
        crate::parser::AbcParser
    }

    /// Returns a new lexer for this language.
    pub fn lexer(&self) -> crate::lexer::AbcLexer {
        crate::lexer::AbcLexer
    }
}
