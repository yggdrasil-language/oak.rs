use oak_core::{TokenType, UniversalTokenRole};

/// Token types for the Mermaid language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MermaidTokenType {
    /// Whitespace characters (spaces and tabs).
    Whitespace,
    /// Newline character.
    ///
    /// 与其他 trivia 不同，换行不会被解析器自动吞掉：flowchart 按行解析，
    /// 需要把行边界暴露给 parser。
    Newline,
    /// Comment.
    Comment,

    /// `graph` / `flowchart` keyword.
    Graph,
    /// Direction (e.g., LR, TD).
    Direction,
    /// Identifier.
    Id,
    /// Node label or edge label, including its delimiters.
    Label,
    /// Connection arrow (e.g., `-->`).
    Arrow,

    /// Lexing error.
    Error,
    /// End of stream marker.
    Eof,
}

#[cfg(feature = "serde")]
impl serde::Serialize for MermaidTokenType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MermaidTokenType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(unsafe { std::mem::transmute(value) })
    }
}

impl TokenType for MermaidTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        // `Newline` 必须保持可见，否则 parser 无法按行切分 flowchart 语句。
        matches!(self, Self::Whitespace | Self::Comment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::Comment => UniversalTokenRole::Comment,
            Self::Graph | Self::Direction => UniversalTokenRole::Keyword,
            Self::Id => UniversalTokenRole::Name,
            Self::Label => UniversalTokenRole::Literal,
            Self::Arrow => UniversalTokenRole::Operator,
            Self::Error => UniversalTokenRole::Error,
            Self::Eof => UniversalTokenRole::Eof,
        }
    }
}
