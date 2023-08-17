use oak_core::{Token, TokenType, UniversalTokenRole};
use std::fmt;

/// A token in the Wolfram language.
pub type WolframToken = Token<WolframTokenType>;

impl fmt::Display for WolframTokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl TokenType for WolframTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment)
    }

    fn role(&self) -> Self::Role {
        // Inherent `role` maps to UniversalTokenRole.
        WolframTokenType::role(self)
    }
}

/// Token types for the Wolfram language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WolframTokenType {
    /// The root token.
    Root,

    // Base tokens
    /// Whitespace.
    Whitespace,
    /// A newline character.
    Newline,

    // Identifiers and literals
    /// An identifier.
    Identifier,
    /// An integer literal.
    Integer,
    /// A real number literal.
    Real,
    /// A string literal.
    String,

    // Keywords
    /// `If` keyword.
    If,
    /// `Then` keyword.
    Then,
    /// `Else` keyword.
    Else,
    /// `While` keyword.
    While,
    /// `For` keyword.
    For,
    /// `Do` keyword.
    Do,
    /// `Function` keyword.
    Function,
    /// `Module` keyword.
    Module,
    /// `Block` keyword.
    Block,
    /// `With` keyword.
    With,
    /// `Table` keyword.
    Table,
    /// `Map` keyword.
    Map,
    /// `Apply` keyword.
    Apply,
    /// `Select` keyword.
    Select,
    /// `Cases` keyword.
    Cases,
    /// `Rule` keyword.
    Rule,
    /// `RuleDelayed` keyword.
    RuleDelayed,
    /// `Set` keyword.
    Set,
    /// `SetDelayed` keyword.
    SetDelayed,
    /// `Unset` keyword.
    Unset,
    /// `Clear` keyword.
    Clear,
    /// `ClearAll` keyword.
    ClearAll,
    /// `Return` keyword.
    Return,
    /// `Break` keyword.
    Break,
    /// `Continue` keyword.
    Continue,
    /// `True` keyword.
    True,
    /// `False` keyword.
    False,
    /// `Null` keyword.
    Null,
    /// `Export` keyword.
    Export,
    /// `Import` keyword.
    Import,

    // Operators
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
    /// `==`.
    Equal,
    /// `!=`.
    NotEqual,
    /// `<`.
    Less,
    /// `>`.
    Greater,
    /// `<=`.
    LessEqual,
    /// `>=`.
    GreaterEqual,
    /// `&&`.
    And,
    /// `||`.
    Or,
    /// `!`.
    Not,

    // Functional operators
    /// `@`.
    At,
    /// `//`.
    SlashSlash,
    /// `/@`.
    MapOperator,
    /// `@@`.
    ApplyOperator,
    /// `@@@`.
    ApplyLevelOperator,
    /// `//@`.
    MapAllOperator,
    /// `&`.
    Ampersand,
    /// `@*`.
    AtStar,
    /// `/*`.
    StarSlash,
    /// `<>`.
    StringJoin,
    /// `:>`.
    RuleDelayedOp,

    // Assignment operators
    /// `=`.
    Assign,
    /// `+=`.
    AddTo,
    /// `-=`.
    SubtractFrom,
    /// `*=`.
    TimesBy,
    /// `/=`.
    DivideBy,

    // Delimiters
    /// `(`.
    LeftParen,
    /// `)`.
    RightParen,
    /// `[`.
    LeftBracket,
    /// `]`.
    RightBracket,
    /// `{`.
    LeftBrace,
    /// `}`.
    RightBrace,
    /// `,`.
    Comma,
    /// `;`.
    Semicolon,
    /// `:`.
    Colon,
    /// `::` MessageName.
    MessageName,
    /// `.`.
    Dot,

    // Special symbols
    /// `->`.
    Arrow,
    /// `=>`.
    DoubleArrow,
    /// `?`.
    Question,
    /// `??` Information.
    DoubleQuestion,
    /// `_`.
    Underscore,
    /// `__`.
    DoubleUnderscore,
    /// `___`.
    TripleUnderscore,
    /// `#`.
    Slot,
    /// `##`.
    SlotSequence,
    /// `!`.
    Factorial,

    // Comments
    /// A comment.
    Comment,

    // Text
    /// Plain text.
    Text,

    /// An error token.
    Error,
    /// End of stream.
    Eof,
}

impl WolframTokenType {
    /// Returns the universal token role for this token type.
    pub fn role(&self) -> UniversalTokenRole {
        match self {
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::Comment => UniversalTokenRole::Comment,
            Self::Identifier => UniversalTokenRole::Name,
            Self::Integer | Self::Real => UniversalTokenRole::Literal,
            Self::String => UniversalTokenRole::Literal,
            Self::LeftParen | Self::LeftBracket | Self::LeftBrace => UniversalTokenRole::Punctuation,
            Self::RightParen | Self::RightBracket | Self::RightBrace => UniversalTokenRole::Punctuation,
            Self::Comma | Self::Semicolon | Self::Colon | Self::Dot => UniversalTokenRole::Punctuation,
            Self::Plus
            | Self::Minus
            | Self::Times
            | Self::Divide
            | Self::Power
            | Self::Equal
            | Self::NotEqual
            | Self::Less
            | Self::Greater
            | Self::LessEqual
            | Self::GreaterEqual
            | Self::And
            | Self::Or
            | Self::Not
            | Self::At
            | Self::SlashSlash
            | Self::MapOperator
            | Self::ApplyOperator
            | Self::ApplyLevelOperator
            | Self::MapAllOperator
            | Self::Ampersand
            | Self::AtStar
            | Self::StarSlash
            | Self::StringJoin
            | Self::RuleDelayedOp
            | Self::Assign
            | Self::AddTo
            | Self::SubtractFrom
            | Self::TimesBy
            | Self::DivideBy
            | Self::Arrow
            | Self::DoubleArrow
            | Self::Question
            | Self::DoubleQuestion
            | Self::MessageName
            | Self::Underscore
            | Self::DoubleUnderscore
            | Self::TripleUnderscore
            | Self::Slot
            | Self::SlotSequence
            | Self::Factorial => UniversalTokenRole::Operator,
            Self::If
            | Self::Then
            | Self::Else
            | Self::While
            | Self::For
            | Self::Do
            | Self::Function
            | Self::Module
            | Self::Block
            | Self::With
            | Self::Table
            | Self::Map
            | Self::Apply
            | Self::Select
            | Self::Cases
            | Self::Rule
            | Self::RuleDelayed
            | Self::Set
            | Self::SetDelayed
            | Self::Unset
            | Self::Clear
            | Self::ClearAll
            | Self::Return
            | Self::Break
            | Self::Continue
            | Self::True
            | Self::False
            | Self::Null
            | Self::Export
            | Self::Import => UniversalTokenRole::Keyword,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            _ => UniversalTokenRole::None,
        }
    }
}
