use oak_core::{Token, TokenType, UniversalTokenRole};
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::fmt;

/// Token alias.
pub type AthenaToken = Token<AthenaTokenType>;

impl fmt::Display for AthenaTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TokenType for AthenaTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    fn role(&self) -> Self::Role {
        self.role()
    }
}

/// Lexeme kinds for athena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum AthenaTokenType {
    /// Root marker.
    Root,
    /// Whitespace.
    Whitespace,
    /// Identifier / variable / function name.
    Identifier,
    /// Integer literal.
    Integer,
    /// Real literal.
    Real,
    /// `+`.
    Plus,
    /// `-`.
    Minus,
    /// `*`.
    Times,
    /// `/`.
    Divide,
    /// `^`.
    Power,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `,`.
    Comma,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `:`.
    Colon,
    /// String literal `"…"`.
    String,
    /// Error token.
    Error,
    /// End of stream.
    Eof,
}

impl AthenaTokenType {
    /// Universal token role.
    pub fn role(&self) -> UniversalTokenRole {
        match self {
            Self::Whitespace => UniversalTokenRole::Whitespace,
            Self::Identifier => UniversalTokenRole::Name,
            Self::Integer | Self::Real | Self::String => UniversalTokenRole::Literal,
            Self::LeftParen | Self::RightParen | Self::Comma
            | Self::LeftBracket | Self::RightBracket | Self::LeftBrace | Self::RightBrace | Self::Colon => {
                UniversalTokenRole::Punctuation
            }
            Self::Plus | Self::Minus | Self::Times | Self::Divide | Self::Power => UniversalTokenRole::Operator,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            Self::Root => UniversalTokenRole::None,
        }
    }
}
