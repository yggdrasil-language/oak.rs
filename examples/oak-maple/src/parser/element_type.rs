use oak_core::{ElementType, UniversalElementRole};
use std::fmt;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MapleElementType {
    Root,
    Expression,
    Call,
    Arguments,
    List,
    Symbol,
    Literal,
    BinaryExpr,
    PrefixExpr,
    Error,
}
impl fmt::Display for MapleElementType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl ElementType for MapleElementType {
    type Role = UniversalElementRole;
    fn role(&self) -> Self::Role {
        UniversalElementRole::None
    }
}
impl From<crate::lexer::token_type::MapleTokenType> for MapleElementType {
    fn from(token: crate::lexer::token_type::MapleTokenType) -> Self {
        match token {
            crate::lexer::token_type::MapleTokenType::Identifier => Self::Symbol,
            crate::lexer::token_type::MapleTokenType::Integer | crate::lexer::token_type::MapleTokenType::Float => Self::Literal,
            _ => Self::Error,
        }
    }
}
