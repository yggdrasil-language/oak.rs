use oak_core::{Token, TokenType, UniversalTokenRole};

/// A VOS token with source range information.
pub type VosToken = Token<VosTokenType>;

/// Token kinds for VOS schema and query syntax.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VosTokenType {
    /// Ignored whitespace.
    Whitespace,
    /// Comments.
    Comment,
    /// End of input.
    Eof,
    /// Identifier.
    Identifier,
    /// `namespace` keyword.
    Namespace,
    /// `table` keyword.
    Table,
    /// `class` keyword.
    Class,
    /// `enums` keyword.
    Enums,
    /// `flags` keyword.
    Flags,
    /// `obsolete` keyword.
    Obsolete,
    /// `using` keyword.
    Using,
    /// `const` keyword.
    Const,
    /// `service` keyword.
    Service,
    /// `micro` keyword.
    Micro,
    /// `macro` keyword.
    Macro,
    /// `query` keyword.
    Query,
    /// `udf` keyword.
    Udf,
    /// `let` keyword.
    Let,
    /// `return` keyword.
    Return,
    /// Boolean literal.
    BooleanLiteral,
    /// Null literal.
    NullLiteral,
    /// String literal.
    StringLiteral,
    /// Number literal.
    NumberLiteral,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `:`.
    Colon,
    /// `;`.
    Semicolon,
    /// `,`.
    Comma,
    /// `=`.
    Equal,
    /// `?`.
    Question,
    /// `<`.
    Less,
    /// `>`.
    Greater,
    /// `.`.
    Dot,
    /// Operator or attribute marker.
    Operator,
    /// Lexical error.
    Error,
}

impl TokenType for VosTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Comment)
    }

    fn is_comment(&self) -> bool {
        matches!(self, Self::Comment)
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, Self::Whitespace)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace => UniversalTokenRole::Whitespace,
            Self::Comment => UniversalTokenRole::Comment,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            Self::Identifier => UniversalTokenRole::Name,
            Self::Operator => UniversalTokenRole::Operator,
            Self::StringLiteral | Self::NumberLiteral | Self::BooleanLiteral | Self::NullLiteral => UniversalTokenRole::Literal,
            Self::LeftBrace | Self::RightBrace | Self::LeftParen | Self::RightParen | Self::LeftBracket | Self::RightBracket | Self::Colon | Self::Semicolon | Self::Comma | Self::Equal | Self::Question | Self::Less | Self::Greater | Self::Dot => {
                UniversalTokenRole::Punctuation
            }
            _ => UniversalTokenRole::Keyword,
        }
    }
}
