//! Token types for the DOT language.
use oak_core::{Token, TokenType, UniversalTokenRole};

/// A token in the DOT language.
pub type DotToken = Token<DotTokenType>;

/// Token types for the DOT language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum DotTokenType {
    // Basic kind
    /// An identifier.
    Identifier,
    /// A string literal.
    String,
    /// A number literal.
    Number,
    /// Whitespace.
    Whitespace,
    /// A newline.
    Newline,

    // DOT keywords
    /// The `graph` keyword.
    Graph,
    /// The `digraph` keyword.
    Digraph,
    /// The `subgraph` keyword.
    Subgraph,
    /// The `node` keyword.
    Node,
    /// The `edge` keyword.
    Edge,
    /// The `strict` keyword.
    Strict,

    // Operators
    /// The `->` arrow operator.
    Arrow, // ->
    /// The `--` line operator.
    Line, // --
    /// The `=` equal operator.
    Equal, // =
    /// The `;` semicolon.
    Semicolon, // ;
    /// The `,` comma.
    Comma, // ,

    // Delimiters
    /// The `{` left brace.
    LeftBrace, // {
    /// The `}` right brace.
    RightBrace, // }
    /// The `[` left bracket.
    LeftBracket, // [
    /// The `]` right bracket.
    RightBracket, // ]
    /// The `(` left paren.
    LeftParen, // (
    /// The `)` right paren.
    RightParen, // )

    // Comments
    /// A comment.
    Comment,

    // Special
    /// The root token.
    Root,
    /// An error token.
    Error,
    /// End of stream marker.
    Eof,
    /// The `:` port separator.
    ///
    /// Appended after `Eof` so every earlier discriminant stays stable for the
    /// `#[repr(u8)]` conversion into [`crate::parser::element_type::DotElementType`].
    Colon,
}

impl TokenType for DotTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Identifier => UniversalTokenRole::Name,
            Self::String | Self::Number => UniversalTokenRole::Literal,
            Self::Graph | Self::Digraph | Self::Subgraph | Self::Node | Self::Edge | Self::Strict => UniversalTokenRole::Keyword,
            Self::Arrow | Self::Line | Self::Equal | Self::Semicolon | Self::Comma | Self::Colon => UniversalTokenRole::Operator,
            Self::LeftBrace | Self::RightBrace | Self::LeftBracket | Self::RightBracket | Self::LeftParen | Self::RightParen => UniversalTokenRole::Punctuation,
            Self::Comment => UniversalTokenRole::Comment,
            _ => UniversalTokenRole::None,
        }
    }
}
