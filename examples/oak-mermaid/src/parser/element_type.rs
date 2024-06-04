use oak_core::{ElementType, UniversalElementRole};

/// Element types for the Mermaid AST.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum MermaidElementType {
    /// Whitespace characters.
    Whitespace,
    /// Newline character.
    Newline,
    /// Comment.
    Comment,

    /// `graph` keyword.
    Graph,
    /// Direction (e.g., LR, TD).
    Direction,
    /// Identifier.
    Id,
    /// Node label.
    Label,
    /// Connection arrow (e.g., `-->`).
    Arrow,

    /// Lexing or parsing error.
    Error,

    /// End of stream.
    ///
    /// 与 [`crate::lexer::token_type::MermaidTokenType::Eof`] 保持 u8 对齐，
    /// 因此 `From<MermaidTokenType>` 可以直接按位转换。
    Eof,

    /// Root node of the diagram.
    Root,
    /// Node in the diagram.
    Node,
    /// Edge between nodes.
    Edge,
    /// `subgraph ... end` block.
    Subgraph,
}

#[cfg(feature = "serde")]
impl serde::Serialize for MermaidElementType {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(*self as u8)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for MermaidElementType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Ok(unsafe { std::mem::transmute(value) })
    }
}

impl MermaidElementType {
    /// Returns `true` if this element type represents a token.
    pub fn is_token(&self) -> bool {
        (*self as u8) <= (Self::Eof as u8) && !self.is_element()
    }

    /// Returns `true` if this element type represents a composite element.
    pub fn is_element(&self) -> bool {
        matches!(self, Self::Root | Self::Node | Self::Edge | Self::Subgraph)
    }
}

impl ElementType for MermaidElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            _ => UniversalElementRole::None,
        }
    }
}

impl From<crate::lexer::token_type::MermaidTokenType> for MermaidElementType {
    fn from(token: crate::lexer::token_type::MermaidTokenType) -> Self {
        unsafe { std::mem::transmute(token) }
    }
}
