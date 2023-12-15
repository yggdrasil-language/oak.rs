use oak_core::{Language, LanguageCategory};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct MapleLanguage;

impl MapleLanguage {
    pub fn new() -> Self {
        Self
    }
}

impl Language for MapleLanguage {
    const NAME: &'static str = "maple";
    const CATEGORY: LanguageCategory = LanguageCategory::Programming;
    type TokenType = crate::lexer::token_type::MapleTokenType;
    type ElementType = crate::parser::element_type::MapleElementType;
    type TypedRoot = crate::ast::MapleRoot;
}
