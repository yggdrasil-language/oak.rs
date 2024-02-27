use crate::lexer::ValkyrieKeywords;
use oak_core::{Token, TokenType, UniversalTokenRole};

/// Alias for `Token<ValkyrieTokenType>`.
pub type ValkyrieToken = Token<ValkyrieTokenType>;
/// Alias for the syntax kind type.
pub type ValkyrieSyntaxKind = ValkyrieTokenType;

/// Token types for the Valkyrie language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ValkyrieTokenType {
    /// End of file token.
    Eof,
    /// Whitespace token.
    Whitespace,
    /// Newline token.
    Newline,
    /// Error token.
    Error,

    // Literals
    /// Integer literal token.
    IntegerLiteral,
    /// Float literal token.
    FloatLiteral,
    /// Boolean literal token.
    BoolLiteral,
    /// String literal token.
    StringLiteral,
    /// String prefix token (e.g., `s`, `f`, `r`, `sql` in `s"..."`).
    StringPrefix,
    /// Character literal token.
    CharLiteral,
    /// Identifier token.
    Identifier,
    /// Label token.
    Label,
    /// TGrammar 模板正文（`%>` 与下一 `<%` 之间的原文，可含 Valkyrie 源码）。
    TemplateText,

    // Comments
    /// Line comment token.
    LineComment,
    /// Block comment token.
    BlockComment,

    // Keywords
    /// Keyword token.
    Keyword(ValkyrieKeywords),
    /// Left parenthesis `(`.
    ParenthesisL,
    /// Right parenthesis `)`.
    ParenthesisR,
    /// Left mathematical angle bracket `⟨` (U+27E8).
    AngleL,
    /// Right mathematical angle bracket `⟩` (U+27E9).
    AngleR,
    /// 左基数索引括号 `⁅` (U+2045)。
    OffsetL,
    /// 右基数索引括号 `⁆` (U+2046)。
    OffsetR,
    /// Left bracket `[`.
    BracketL,
    /// Right bracket `]`.
    BracketR,
    /// Left brace `{`.
    BraceL,
    /// Right brace `}`.
    BraceR,
    /// TGrammar 控制块开始 `<%`。
    TemplateL,
    /// TGrammar 控制块结束 `%>`。
    TemplateR,

    // Operators & Punctuation
    /// Ampersand operator `&`.
    Ampersand,
    /// Logical and operator `&&`.
    AndAnd,
    /// Arrow operator `->`.
    Arrow,
    /// At symbol `@`.
    At,
    /// Bang operator `!`.
    Bang,
    /// Caret operator `^`.
    Caret,
    /// Colon operator `:`.
    Colon,
    /// Path separator `::`.
    ColonColon,
    /// Assignment operator `:=`.
    ColonEq,
    /// Comma separator `,`.
    Comma,
    /// Dollar symbol `$`.
    Dollar,
    /// Dot operator `.`.
    Dot,
    /// Range operator `..`.
    DotDot,
    /// Assignment operator `=`.
    Eq,
    /// Equality operator `==`.
    EqEq,
    /// Greater than or equal operator `>=`.
    GreaterEq,
    /// Greater than operator `>`.
    GreaterThan,
    /// Left shift operator `<<`.
    LeftShift,
    /// Less than or equal operator `<=`.
    LessEq,
    /// Less than operator `<`.
    LessThan,
    /// Minus operator `-`.
    Minus,
    /// Not equal operator `!=`.
    NotEq,
    /// Logical or operator `||`.
    OrOr,
    /// Percent operator `%`.
    Percent,
    /// Pipe operator `|`.
    Pipe,
    /// Plus operator `+`.
    Plus,
    /// Question operator `?`.
    Question,
    /// Right shift operator `>>`.
    RightShift,
    /// Semicolon separator `;`.
    Semicolon,
    /// Slash operator `/`.
    Slash,
    /// Star operator `*`.
    Star,
    /// Tilde operator `~`.
    Tilde,
    /// Underscore token `_`.
    Underscore,
}

impl TokenType for ValkyrieTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::LineComment | Self::BlockComment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::LineComment | Self::BlockComment => UniversalTokenRole::Comment,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            Self::Keyword(_) => UniversalTokenRole::Keyword,
            Self::Identifier | Self::StringPrefix => UniversalTokenRole::Name,
            Self::StringLiteral | Self::CharLiteral => UniversalTokenRole::Literal,
            Self::IntegerLiteral | Self::FloatLiteral | Self::BoolLiteral => UniversalTokenRole::Literal,
            _ => UniversalTokenRole::None,
        }
    }
}
