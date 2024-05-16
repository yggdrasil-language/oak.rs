use oak_core::{Source, Token, TokenType, UniversalElementRole, UniversalTokenRole};

/// A token in a Typst document.
pub type TypstToken = Token<TypstTokenType>;

impl TokenType for TypstTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        // Typst 是行导向的标记语言，换行决定标题、列表项与段落边界，
        // 因此 `Newline` 必须保留给解析器，不能与空白、注释一起作为 trivia 跳过。
        matches!(self, Self::Whitespace | Self::LineComment | Self::BlockComment)
    }

    fn role(&self) -> Self::Role {
        match self {
            Self::Whitespace | Self::Newline => UniversalTokenRole::Whitespace,
            Self::LineComment | Self::BlockComment => UniversalTokenRole::Comment,
            Self::Let | Self::If | Self::Else | Self::For | Self::While | Self::Break | Self::Continue | Self::Return | Self::Import | Self::Include | Self::Set | Self::Show => UniversalTokenRole::Keyword,
            Self::True | Self::False | Self::StringLiteral | Self::NumericLiteral => UniversalTokenRole::Literal,
            Self::Identifier => UniversalTokenRole::Name,
            Self::Plus | Self::Minus | Self::Star | Self::Slash | Self::Percent | Self::Equal | Self::EqualEqual | Self::NotEqual | Self::Less | Self::Greater | Self::LessEqual | Self::GreaterEqual | Self::And | Self::Or | Self::Not => {
                UniversalTokenRole::Operator
            }
            Self::LeftParen
            | Self::RightParen
            | Self::LeftBrace
            | Self::RightBrace
            | Self::LeftBracket
            | Self::RightBracket
            | Self::Semicolon
            | Self::Comma
            | Self::Dot
            | Self::Colon
            | Self::Hash
            | Self::At
            | Self::Dollar
            | Self::Underscore
            | Self::Backtick => UniversalTokenRole::Punctuation,
            Self::Eof => UniversalTokenRole::Eof,
            Self::Error => UniversalTokenRole::Error,
            _ => UniversalTokenRole::None,
        }
    }
}

/// Represents the type of a Typst token.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TypstTokenType {
    /// Root node.
    Root,
    /// A document.
    Document,
    /// A block of content.
    Block,

    /// A heading (= Heading).
    Heading,
    /// A paragraph of text.
    Paragraph,
    /// A list (+ Item).
    List,
    /// A list item.
    ListItem,
    /// An enumeration item (1. Item).
    EnumItem,
    /// A table.
    Table,
    /// A table row.
    TableRow,
    /// A table cell.
    TableCell,
    /// A figure.
    Figure,
    /// An image.
    Image,
    /// A link.
    Link,

    /// Plain text.
    Text,
    /// Strong text (*strong*).
    Strong,
    /// Emphasized text (_emphasis_).
    Emphasis,
    /// Code content (`code`).
    Code,
    /// Math content ($...$).
    Math,
    /// Inline math content.
    InlineMath,
    /// Display math content.
    DisplayMath,
    /// Raw content.
    Raw,
    /// A quote.
    Quote,

    /// A script.
    Script,
    /// An expression.
    Expression,
    /// A function call.
    FunctionCall,
    /// A variable.
    Variable,
    /// An assignment.
    Assignment,
    /// A conditional statement.
    Conditional,
    /// A loop statement.
    Loop,
    /// An import statement.
    Import,
    /// An include statement.
    Include,

    /// A set rule.
    Set,
    /// A show rule.
    Show,
    /// A style rule.
    Style,
    /// A color.
    Color,
    /// A font.
    Font,
    /// A size.
    Size,

    /// 'let' keyword.
    Let,
    /// 'if' keyword.
    If,
    /// 'else' keyword.
    Else,
    /// 'for' keyword.
    For,
    /// 'while' keyword.
    While,
    /// 'break' keyword.
    Break,
    /// 'continue' keyword.
    Continue,
    /// 'return' keyword.
    Return,
    /// 'true' keyword.
    True,
    /// 'false' keyword.
    False,

    /// Plus operator (+).
    Plus,
    /// Minus operator (-).
    Minus,
    /// Multiplication operator (*).
    Star,
    /// Division operator (/).
    Slash,
    /// Modulo operator (%).
    Percent,
    /// Assignment operator (=).
    Equal,
    /// Equality operator (==).
    EqualEqual,
    /// Inequality operator (!=).
    NotEqual,
    /// Less than operator (<).
    Less,
    /// Greater than operator (>).
    Greater,
    /// Less than or equal to operator (<=).
    LessEqual,
    /// Greater than or equal to operator (>=).
    GreaterEqual,
    /// Logical AND operator (and).
    And,
    /// Logical OR operator (or).
    Or,
    /// Logical NOT operator (not).
    Not,

    /// Left parenthesis (().
    LeftParen,
    /// Right parenthesis ()).
    RightParen,
    /// Left brace ({).
    LeftBrace,
    /// Right brace (}).
    RightBrace,
    /// Left bracket ([).
    LeftBracket,
    /// Right bracket (]).
    RightBracket,
    /// Semicolon separator (;).
    Semicolon,
    /// Comma separator (,).
    Comma,
    /// Dot separator (.).
    Dot,
    /// Colon separator (:).
    Colon,
    /// Hash symbol (#).
    Hash,
    /// At symbol (@).
    At,
    /// Dollar symbol ($).
    Dollar,
    /// Underscore symbol (_).
    Underscore,
    /// Backtick symbol (`).
    Backtick,

    /// A string literal ("...").
    StringLiteral,
    /// A numeric literal.
    NumericLiteral,
    /// An identifier.
    Identifier,

    /// A line comment (// ...).
    LineComment,
    /// A block comment (/* ... */).
    BlockComment,
    /// Whitespace characters.
    Whitespace,
    /// A newline character.
    Newline,

    /// End of file marker.
    Eof,
    /// Error token.
    Error,
}
