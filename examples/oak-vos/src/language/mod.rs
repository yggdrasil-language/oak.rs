use oak_core::{Language, LanguageCategory};

/// Oak language configuration for VOS.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct VosLanguage;

impl Language for VosLanguage {
    const NAME: &'static str = "vos";
    const CATEGORY: LanguageCategory = LanguageCategory::Dsl;

    type TokenType = crate::lexer::VosTokenType;
    type ElementType = crate::parser::VosElementType;
    type TypedRoot = crate::VosRoot;
}
