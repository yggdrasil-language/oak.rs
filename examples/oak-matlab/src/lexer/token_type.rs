use oak_core::{Token, TokenType, UniversalTokenRole};

/// Token for the Matlab language.
pub type MatlabToken = Token<MatlabTokenType>;

impl MatlabTokenType {
    /// Returns true if the token type is a token.
    pub fn is_token(&self) -> bool {
        !self.is_element()
    }

    /// Returns true if the token type is an element.
    pub fn is_element(&self) -> bool {
        matches!(self, Self::Script | Self::FunctionDef | Self::ClassDef | Self::Block | Self::Expression | Self::Statement)
    }
}

impl TokenType for MatlabTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, Self::Whitespace | Self::Newline | Self::Comment | Self::BlockComment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::Comment | Self::BlockComment => UniversalTokenRole::Comment,
            Self::Identifier => UniversalTokenRole::Name,
            Self::Number | Self::String | Self::Character => UniversalTokenRole::Literal,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            _ => UniversalTokenRole::None,
        }
    }
}

/// Token types for the Matlab language.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum MatlabTokenType {
    // Basic tokens
    /// Whitespace.
    Whitespace,
    /// Newline.
    Newline,
    /// Line comment.
    Comment,
    /// Block comment.
    BlockComment,

    // Identifiers and literals
    /// Identifier.
    Identifier,
    /// Number literal.
    Number,
    /// String literal.
    String,
    /// Character literal.
    Character,

    // Keywords
    /// `function` keyword.
    Function,
    /// `end` keyword.
    End,
    /// `if` keyword.
    If,
    /// `else` keyword.
    Else,
    /// `elseif` keyword.
    Elseif,
    /// `while` keyword.
    While,
    /// `for` keyword.
    For,
    /// `parfor` keyword.
    Parfor,
    /// `spmd` keyword.
    Spmd,
    /// `break` keyword.
    Break,
    /// `continue` keyword.
    Continue,
    /// `return` keyword.
    Return,
    /// `switch` keyword.
    Switch,
    /// `case` keyword.
    Case,
    /// `otherwise` keyword.
    Otherwise,
    /// `try` keyword.
    Try,
    /// `catch` keyword.
    Catch,
    /// `global` keyword.
    Global,
    /// `persistent` keyword.
    Persistent,
    /// `classdef` keyword.
    Classdef,
    /// `properties` keyword.
    Properties,
    /// `methods` keyword.
    Methods,
    /// `events` keyword.
    Events,

    // Operators
    /// `+` operator.
    Plus, // +
    /// `-` operator.
    Minus, // -
    /// `*` operator.
    Times, // *
    /// `/` operator.
    Divide, // /
    /// `^` operator.
    Power, // ^
    /// `\` operator.
    LeftDivide, // \
    /// `.*` operator.
    DotTimes, // .*
    /// `./` operator.
    DotDivide, // ./
    /// `.^` operator.
    DotPower, // .^
    /// `.\` operator.
    DotLeftDivide, // .\

    // Comparison operators
    /// `==` operator.
    Equal, // ==
    /// `~=` operator.
    NotEqual, // ~=
    /// `<` operator.
    Less, // <
    /// `>` operator.
    Greater, // >
    /// `<=` operator.
    LessEqual, // <=
    /// `>=` operator.
    GreaterEqual, // >=

    // Logical operators
    /// `&` operator.
    And, // &
    /// `|` operator.
    Or, // |
    /// `~` operator.
    Not, // ~
    /// `&&` operator.
    AndAnd, // &&
    /// `||` operator.
    OrOr, // ||

    // Assignment operators
    /// `=` operator.
    Assign, // =

    // Delimiters
    /// `(` delimiter.
    LeftParen, // (
    /// `)` delimiter.
    RightParen, // )
    /// `[` delimiter.
    LeftBracket, // [
    /// `]` delimiter.
    RightBracket, // ]
    /// `{` delimiter.
    LeftBrace, // {
    /// `}` delimiter.
    RightBrace, // }
    /// `;` delimiter.
    Semicolon, // ;
    /// `,` delimiter.
    Comma, // ,
    /// `.` delimiter.
    Dot, // .
    /// `:` delimiter.
    Colon, // :
    /// `?` delimiter.
    Question, // ?
    /// `@` delimiter.
    At, // @

    // Special operators
    /// `'` operator.
    Transpose, // '
    /// `.'` operator.
    DotTranspose, // .'

    // Generalized types
    /// General operator.
    Operator,
    /// General delimiter.
    Delimiter,

    // Error handling
    /// Error token.
    Error,

    // Document structure
    /// Script element.
    Script,
    /// Function definition element.
    FunctionDef,
    /// Class definition element.
    ClassDef,
    /// Block element.
    Block,
    /// Expression element.
    Expression,
    /// Statement element.
    Statement,

    // EOF
    /// End of stream.
    Eof,
}
