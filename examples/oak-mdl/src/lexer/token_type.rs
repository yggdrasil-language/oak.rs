//! Token types for MDL connection tables.

use oak_core::language::{TokenType, UniversalTokenRole};

/// Token kinds emitted by [`crate::lexer::MdlLexer`].
///
/// Every kind describes one physical line (the trailing newline belongs to the
/// token). The lexer deliberately keeps `Line` generic: column meaning is
/// positional and is resolved by the builder after the counts line is known.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MdlTokenType {
    /// A content-bearing line whose meaning depends on its position.
    Line,
    /// An `M  XXX` property line.
    Property,
    /// A `> <NAME>` data-field header line.
    DataHeader,
    /// A `$$$$` record separator.
    RecordSeparator,
    /// A `$RXN`, `$RDFILE`, or `$MOL` reaction marker.
    ReactionMarker,
    /// End of input.
    Eof,
}

impl TokenType for MdlTokenType {
    type Role = UniversalTokenRole;

    const END_OF_STREAM: Self = Self::Eof;

    fn role(&self) -> Self::Role {
        match self {
            Self::Line => UniversalTokenRole::Literal,
            Self::Property => UniversalTokenRole::Keyword,
            Self::DataHeader => UniversalTokenRole::Keyword,
            Self::RecordSeparator => UniversalTokenRole::Punctuation,
            Self::ReactionMarker => UniversalTokenRole::Keyword,
            Self::Eof => UniversalTokenRole::Eof,
        }
    }
}
