//! Language definition for SMILES.

use oak_core::language::{Language, LanguageCategory};

use crate::{
    ast::SmilesRoot, lexer::token_type::SmilesTokenType, parser::element_type::SmilesElementType,
};

/// Language definition for SMILES and reaction SMILES.
pub struct SmilesLanguage;

impl Language for SmilesLanguage {
    const NAME: &'static str = "smiles";
    const CATEGORY: LanguageCategory = LanguageCategory::Dsl;

    type TokenType = SmilesTokenType;
    type ElementType = SmilesElementType;
    type TypedRoot = SmilesRoot;
}

impl Default for SmilesLanguage {
    fn default() -> Self {
        Self
    }
}

impl SmilesLanguage {
    /// Returns a new parser for this language.
    pub fn parser(&self) -> crate::parser::SmilesParser {
        crate::parser::SmilesParser
    }

    /// Returns a new lexer for this language.
    pub fn lexer(&self) -> crate::lexer::SmilesLexer {
        crate::lexer::SmilesLexer
    }
}
