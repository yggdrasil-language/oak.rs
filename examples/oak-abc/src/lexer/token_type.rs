//! Token types for ABC notation.

use oak_core::language::{TokenType, UniversalTokenRole};

/// Token kinds emitted by [`crate::lexer::AbcLexer`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AbcTokenType {
    /// One physical line, with its trailing newline.
    ///
    /// ABC is line-oriented: a header field, a music line, a lyric line, and a
    /// directive each occupy a whole line, so the line is the natural token.
    Line,
    /// End of input.
    Eof,
}

impl TokenType for AbcTokenType {
    type Role = UniversalTokenRole;

    const END_OF_STREAM: Self = Self::Eof;

    fn role(&self) -> Self::Role {
        match self {
            Self::Line => UniversalTokenRole::Literal,
            Self::Eof => UniversalTokenRole::Eof,
        }
    }
}
