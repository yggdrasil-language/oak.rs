//! Element types for the MDL syntax tree.

use oak_core::language::{ElementType, UniversalElementRole};

use crate::lexer::token_type::MdlTokenType;

/// Element kinds of the MDL green tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MdlElementType {
    /// Whole document.
    SourceFile,
    /// One header line (name, program, or comment).
    HeaderLine,
    /// The counts line.
    CountsLine,
    /// One atom line.
    AtomLine,
    /// One bond line.
    BondLine,
    /// One `M  XXX` property line.
    PropertyLine,
    /// A `> <NAME>` data-field header line.
    DataFieldName,
    /// One line of a data-field value.
    DataValue,
    /// A `$$$$` record separator.
    RecordSeparator,
    /// A reaction marker line.
    ReactionMarker,
}

impl ElementType for MdlElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::SourceFile => UniversalElementRole::Root,
            Self::HeaderLine => UniversalElementRole::Documentation,
            Self::CountsLine => UniversalElementRole::Metadata,
            Self::AtomLine | Self::BondLine => UniversalElementRole::Definition,
            Self::PropertyLine => UniversalElementRole::Attribute,
            Self::DataFieldName => UniversalElementRole::AttributeKey,
            Self::DataValue => UniversalElementRole::Attribute,
            Self::RecordSeparator => UniversalElementRole::Container,
            Self::ReactionMarker => UniversalElementRole::Metadata,
        }
    }
}

impl From<MdlTokenType> for MdlElementType {
    fn from(kind: MdlTokenType) -> Self {
        match kind {
            MdlTokenType::Line => Self::HeaderLine,
            MdlTokenType::Property => Self::PropertyLine,
            MdlTokenType::DataHeader => Self::DataFieldName,
            MdlTokenType::RecordSeparator => Self::RecordSeparator,
            MdlTokenType::ReactionMarker => Self::ReactionMarker,
            MdlTokenType::Eof => Self::HeaderLine,
        }
    }
}
