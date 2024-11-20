//! Element types for the SMILES syntax tree.

use oak_core::language::{ElementType, UniversalElementRole};

use crate::lexer::token_type::SmilesTokenType;

/// Element kinds of the SMILES green tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SmilesElementType {
    /// Whole document.
    SourceFile,
    /// An unbracketed organic atom.
    OrganicAtom,
    /// A bracketed atom.
    BracketAtom,
    /// A bond symbol.
    Bond,
    /// A branch opening.
    BranchOpen,
    /// A branch closing.
    BranchClose,
    /// A fragment separator.
    Dot,
    /// A ring label.
    RingLabel,
    /// A reaction arrow.
    Arrow,
}

impl ElementType for SmilesElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::SourceFile => UniversalElementRole::Root,
            Self::OrganicAtom | Self::BracketAtom => UniversalElementRole::Definition,
            Self::Bond => UniversalElementRole::Attribute,
            Self::BranchOpen | Self::BranchClose => UniversalElementRole::Container,
            Self::Dot | Self::Arrow => UniversalElementRole::Container,
            Self::RingLabel => UniversalElementRole::Reference,
        }
    }
}

impl From<SmilesTokenType> for SmilesElementType {
    fn from(kind: SmilesTokenType) -> Self {
        match kind {
            SmilesTokenType::OrganicAtom => Self::OrganicAtom,
            SmilesTokenType::BracketAtom => Self::BracketAtom,
            SmilesTokenType::Bond => Self::Bond,
            SmilesTokenType::BranchOpen => Self::BranchOpen,
            SmilesTokenType::BranchClose => Self::BranchClose,
            SmilesTokenType::Dot => Self::Dot,
            SmilesTokenType::RingLabel => Self::RingLabel,
            SmilesTokenType::Arrow => Self::Arrow,
            SmilesTokenType::Eof => Self::Dot,
        }
    }
}
