//! Token types for the PlantUML class and sequence subset.
//!
//! Whitespace is the only ignored token. Newlines, comments and every statement
//! token stay in the token stream because the parser is line-oriented.

use oak_core::{Token, TokenType, UniversalTokenRole};

/// A token in the PlantUML subset.
pub type PlantUmlToken = Token<PlantUmlTokenType>;

/// Token types for the PlantUML class and sequence subset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlantUmlTokenType {
    /// Spaces and tabs.
    Whitespace,
    /// A line break.
    Newline,
    /// A line comment beginning with `'`.
    Comment,

    /// The `@startuml` marker.
    StartMarker,
    /// The `@enduml` marker.
    EndMarker,

    /// The `class` keyword.
    Class,
    /// The `interface` keyword.
    Interface,
    /// The `abstract` keyword.
    Abstract,
    /// The `enum` keyword.
    Enum,
    /// The `participant` keyword.
    Participant,
    /// The `actor` keyword.
    Actor,
    /// The `activate` keyword.
    Activate,
    /// The `deactivate` keyword.
    Deactivate,
    /// The `as` alias keyword.
    As,

    /// A class relation arrow or a sequence message arrow.
    Arrow,
    /// The `{` member-block opener.
    LeftBrace,
    /// The `}` member-block closer.
    RightBrace,
    /// The `(` method opener.
    LeftParen,
    /// The `)` method closer.
    RightParen,
    /// The `+` public visibility marker.
    Plus,
    /// The `-` private visibility marker.
    Minus,
    /// The `#` protected visibility marker.
    Hash,
    /// The `:` label separator.
    Colon,

    /// A name such as a class or participant identifier.
    Identifier,
    /// A double-quoted string such as a display name.
    String,
    /// Any other non-whitespace text.
    Text,
    /// A malformed token.
    Error,
    /// End of input.
    Eof,
}

impl TokenType for PlantUmlTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Identifier | Self::Text => UniversalTokenRole::Name,
            Self::String => UniversalTokenRole::Literal,
            Self::Class | Self::Interface | Self::Abstract | Self::Enum | Self::Participant | Self::Actor | Self::Activate | Self::Deactivate | Self::As => UniversalTokenRole::Keyword,
            Self::Arrow | Self::Colon | Self::Plus | Self::Minus | Self::Hash => UniversalTokenRole::Operator,
            Self::LeftBrace | Self::RightBrace | Self::LeftParen | Self::RightParen => UniversalTokenRole::Punctuation,
            Self::StartMarker | Self::EndMarker => UniversalTokenRole::Keyword,
            Self::Comment => UniversalTokenRole::Comment,
            Self::Whitespace => UniversalTokenRole::Whitespace,
            Self::Error => UniversalTokenRole::Error,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Newline => UniversalTokenRole::Whitespace,
        }
    }
}
