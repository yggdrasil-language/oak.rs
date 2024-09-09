//! PlantUML language configuration.

use oak_core::{Language, LanguageCategory};

/// PlantUML language configuration for the class and sequence subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlantUmlLanguage;

impl PlantUmlLanguage {
    /// Creates a PlantUML language configuration.
    pub fn new() -> Self {
        Self
    }
}

impl Default for PlantUmlLanguage {
    fn default() -> Self {
        Self::new()
    }
}

impl Language for PlantUmlLanguage {
    const NAME: &'static str = "plantuml";
    const CATEGORY: LanguageCategory = LanguageCategory::Modeling;

    type TokenType = crate::lexer::token_type::PlantUmlTokenType;
    type ElementType = crate::parser::element_type::PlantUmlElementType;
    type TypedRoot = crate::ast::PlantUmlRoot;
}
