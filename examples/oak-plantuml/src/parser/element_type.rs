//! Element types for the PlantUML parser's green tree.

use crate::lexer::token_type::PlantUmlTokenType;
use oak_core::{ElementType, UniversalElementRole};

/// Element types for the PlantUML class and sequence subset.
///
/// Token-level variants exist so the builder can inspect leaves directly. The
/// composite variants group each statement into a single green-tree element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlantUmlElementType {
    /// The document root.
    Root,
    /// Whitespace and newlines.
    Trivia,
    /// The `@startuml` marker.
    StartMarker,
    /// The `@enduml` marker.
    EndMarker,
    /// A class, interface, abstract class or enum declaration.
    ClassDeclaration,
    /// A `{ ... }` member block.
    ClassMemberBlock,
    /// One class member line.
    ClassMember,
    /// A class relation line.
    ClassRelation,
    /// A `participant` or `actor` declaration.
    ParticipantDeclaration,
    /// A sequence message line.
    SequenceMessage,
    /// An `activate` statement.
    Activate,
    /// A `deactivate` statement.
    Deactivate,
    /// An activity diagram container.
    ActivityDiagram,
    /// One activity statement line.
    ActivityStatement,
    /// A state diagram container.
    StateDiagram,
    /// One state statement line.
    StateStatement,
    /// A component diagram container.
    ComponentDiagram,
    /// One component statement line.
    ComponentStatement,
    /// A line comment.
    Comment,

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
    /// The `as` keyword.
    As,
    /// A relation or message arrow.
    Arrow,
    /// The `{` token.
    LeftBrace,
    /// The `}` token.
    RightBrace,
    /// The `(` token.
    LeftParen,
    /// The `)` token.
    RightParen,
    /// The `+` token.
    Plus,
    /// The `-` token.
    Minus,
    /// The `#` token.
    Hash,
    /// The `~` token.
    Tilde,
    /// The `:` token.
    Colon,
    /// An identifier.
    Identifier,
    /// A quoted string.
    String,
    /// Raw text.
    Text,
    /// A malformed token.
    Error,
    /// End of file.
    Eof,
}

impl ElementType for PlantUmlElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            Self::ClassDeclaration | Self::ParticipantDeclaration => UniversalElementRole::Definition,
            Self::ClassMemberBlock | Self::ActivityDiagram | Self::StateDiagram | Self::ComponentDiagram => UniversalElementRole::Container,
            Self::ClassRelation | Self::SequenceMessage | Self::Activate | Self::Deactivate | Self::ActivityStatement | Self::StateStatement | Self::ComponentStatement => UniversalElementRole::Statement,
            Self::ClassMember => UniversalElementRole::Attribute,
            Self::Comment => UniversalElementRole::Documentation,
            Self::StartMarker | Self::EndMarker | Self::Trivia => UniversalElementRole::Metadata,
            Self::Class | Self::Interface | Self::Abstract | Self::Enum | Self::Participant | Self::Actor | Self::As => UniversalElementRole::Metadata,
            Self::Arrow | Self::LeftBrace | Self::RightBrace | Self::LeftParen | Self::RightParen | Self::Plus | Self::Minus | Self::Hash | Self::Tilde | Self::Colon | Self::Identifier | Self::String | Self::Text | Self::Error | Self::Eof => {
                UniversalElementRole::Metadata
            }
        }
    }
}

impl From<PlantUmlTokenType> for PlantUmlElementType {
    fn from(kind: PlantUmlTokenType) -> Self {
        match kind {
            PlantUmlTokenType::Whitespace | PlantUmlTokenType::Newline => Self::Trivia,
            PlantUmlTokenType::Comment => Self::Comment,
            PlantUmlTokenType::StartMarker => Self::StartMarker,
            PlantUmlTokenType::EndMarker => Self::EndMarker,
            PlantUmlTokenType::Class => Self::Class,
            PlantUmlTokenType::Interface => Self::Interface,
            PlantUmlTokenType::Abstract => Self::Abstract,
            PlantUmlTokenType::Enum => Self::Enum,
            PlantUmlTokenType::Participant => Self::Participant,
            PlantUmlTokenType::Actor => Self::Actor,
            PlantUmlTokenType::Activate => Self::Activate,
            PlantUmlTokenType::Deactivate => Self::Deactivate,
            PlantUmlTokenType::As => Self::As,
            PlantUmlTokenType::Arrow => Self::Arrow,
            PlantUmlTokenType::LeftBrace => Self::LeftBrace,
            PlantUmlTokenType::RightBrace => Self::RightBrace,
            PlantUmlTokenType::LeftParen => Self::LeftParen,
            PlantUmlTokenType::RightParen => Self::RightParen,
            PlantUmlTokenType::Plus => Self::Plus,
            PlantUmlTokenType::Minus => Self::Minus,
            PlantUmlTokenType::Hash => Self::Hash,
            PlantUmlTokenType::Tilde => Self::Tilde,
            PlantUmlTokenType::Colon => Self::Colon,
            PlantUmlTokenType::Identifier => Self::Identifier,
            PlantUmlTokenType::String => Self::String,
            PlantUmlTokenType::Text => Self::Text,
            PlantUmlTokenType::Error => Self::Error,
            PlantUmlTokenType::Eof => Self::Eof,
        }
    }
}
