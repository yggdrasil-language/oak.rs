//! Element types for the ABC syntax tree.

use oak_core::language::{ElementType, UniversalElementRole};

use crate::lexer::token_type::AbcTokenType;

/// Element kinds of the ABC green tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbcElementType {
    /// Whole document.
    SourceFile,
    /// One physical line.
    Line,
}

impl ElementType for AbcElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::SourceFile => UniversalElementRole::Root,
            Self::Line => UniversalElementRole::Documentation,
        }
    }
}

impl From<AbcTokenType> for AbcElementType {
    fn from(kind: AbcTokenType) -> Self {
        match kind {
            AbcTokenType::Line => Self::Line,
            AbcTokenType::Eof => Self::Line,
        }
    }
}
