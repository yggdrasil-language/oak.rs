use oak_core::{Token, TokenType, UniversalTokenRole};

pub type MapleToken = Token<MapleTokenType>;

impl TokenType for MapleTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;
    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }
    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace => UniversalTokenRole::Whitespace,
            Self::Comment => UniversalTokenRole::Comment,
            Self::Identifier => UniversalTokenRole::Name,
            Self::Integer | Self::Float => UniversalTokenRole::Literal,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            _ => UniversalTokenRole::None,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MapleTokenType {
    Whitespace,
    Comment,
    Identifier,
    Integer,
    Float,
    Plus,
    Minus,
    Times,
    Divide,
    Power,
    Equal,
    LeftParen,
    RightParen,
    LeftBracket,
    RightBracket,
    Comma,
    Semicolon,
    Eof,
    Error,
}
