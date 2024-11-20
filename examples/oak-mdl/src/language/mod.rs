//! Language definition for MDL connection tables.

use oak_core::language::{Language, LanguageCategory};

use crate::{
    ast::MdlRoot, lexer::token_type::MdlTokenType, parser::element_type::MdlElementType,
};

/// Language definition for MDL molfiles and SDF members.
pub struct MdlLanguage;

impl Language for MdlLanguage {
    const NAME: &'static str = "mdl";
    const CATEGORY: LanguageCategory = LanguageCategory::Config;

    type TokenType = MdlTokenType;
    type ElementType = MdlElementType;
    type TypedRoot = MdlRoot;
}

impl Default for MdlLanguage {
    fn default() -> Self {
        Self
    }
}

impl MdlLanguage {
    /// Returns a new parser for this language.
    pub fn parser(&self) -> crate::parser::MdlParser {
        crate::parser::MdlParser
    }

    /// Returns a new lexer for this language.
    pub fn lexer(&self) -> crate::lexer::MdlLexer {
        crate::lexer::MdlLexer
    }
}
