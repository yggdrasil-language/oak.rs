use oak_core::{Token, UniversalTokenRole};

/// Represents a token in an SQL source file.
pub type SqlToken = Token<SqlTokenType>;

impl oak_core::TokenType for SqlTokenType {
    type Role = UniversalTokenRole;
    const END_OF_STREAM: Self = Self::Eof;

    fn is_ignored(&self) -> bool {
        matches!(self, SqlTokenType::Whitespace | SqlTokenType::Newline | SqlTokenType::Comment | SqlTokenType::LineComment | SqlTokenType::BlockComment)
    }

    fn is_comment(&self) -> bool {
        matches!(self, SqlTokenType::Comment | SqlTokenType::LineComment | SqlTokenType::BlockComment)
    }

    fn is_whitespace(&self) -> bool {
        matches!(self, SqlTokenType::Whitespace | SqlTokenType::Newline)
    }

    fn role(&self) -> Self::Role {
        use UniversalTokenRole::*;
        match self {
            Self::Whitespace | Self::Newline => Whitespace,
            Self::Comment | Self::LineComment | Self::BlockComment => Comment,
            Self::NumberLiteral | Self::FloatLiteral | Self::StringLiteral | Self::BooleanLiteral | Self::NullLiteral => Literal,
            Self::Identifier_ => Name,
            Self::Select
            | Self::From
            | Self::Where
            | Self::Insert
            | Self::Into
            | Self::Values
            | Self::Update
            | Self::Set
            | Self::Delete
            | Self::Create
            | Self::Table
            | Self::Drop
            | Self::Alter
            | Self::Add
            | Self::Column
            | Self::Primary
            | Self::Key
            | Self::Foreign
            | Self::References
            | Self::Index
            | Self::Unique
            | Self::Not
            | Self::Null
            | Self::Default
            | Self::AutoIncrement
            | Self::And
            | Self::Or
            | Self::In
            | Self::Like
            | Self::Between
            | Self::Is
            | Self::As
            | Self::Join
            | Self::Inner
            | Self::Left
            | Self::Right
            | Self::Full
            | Self::Outer
            | Self::On
            | Self::Group
            | Self::By
            | Self::Having
            | Self::Order
            | Self::Asc
            | Self::Desc
            | Self::Limit
            | Self::Offset
            | Self::Union
            | Self::All
            | Self::Distinct
            | Self::Count
            | Self::Sum
            | Self::Avg
            | Self::Min
            | Self::Explain
            | Self::Vector
            | Self::View
            | Self::Database
            | Self::Schema
            | Self::True
            | Self::False
            | Self::Exists
            | Self::Trigger
            | Self::After
            | Self::Delimiter
            | Self::For
            | Self::Each
            | Self::Row
            | Self::Check
            | Self::Rename
            | Self::To
            | Self::Case
            | Self::When
            | Self::Then
            | Self::Else
            | Self::End
            | Self::If
            | Self::Begin
            | Self::Commit
            | Self::Rollback
            | Self::Transaction
            | Self::Conflict
            | Self::Do
            | Self::Nothing
            | Self::Returning
            | Self::Ilike
            | Self::Strict
            | Self::Without
            | Self::Rowid => Keyword,
            Self::Int | Self::Integer | Self::Varchar | Self::Char | Self::Text | Self::Date | Self::Time | Self::Timestamp | Self::Decimal | Self::Float | Self::Double | Self::Boolean | Self::Serial | Self::BigSerial => Keyword, /* Types are often keywords */
            Self::Plus | Self::Minus | Self::Star | Self::Slash | Self::Percent | Self::Equal | Self::NotEqual | Self::Less | Self::Greater | Self::LessEqual | Self::GreaterEqual | Self::Concat => Operator,
            Self::LeftParen | Self::RightParen | Self::LeftBracket | Self::RightBracket | Self::LeftBrace | Self::RightBrace | Self::Comma | Self::Semicolon | Self::Dot | Self::Colon | Self::Question => Punctuation,
            Self::Error => Error,
            _ => UniversalTokenRole::None,
        }
    }
}

/// Token types for the SQL language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SqlTokenType {
    /// Root node.
    Root,
    /// EXPLAIN statement.
    ExplainStatement,
    /// TRANSACTION statement.
    TransactionStatement,
    /// PRAGMA statement.
    PragmaStatement,
    /// SHOW statement.
    ShowStatement,
    /// SELECT statement.
    SelectStatement,
    /// INSERT statement.
    InsertStatement,
    /// UPDATE statement.
    UpdateStatement,
    /// DELETE statement.
    DeleteStatement,
    /// CREATE statement.
    CreateStatement,
    /// DROP statement.
    DropStatement,
    /// ALTER statement.
    AlterStatement,
    /// SQL expression.
    Expression,
    /// Identifier.
    Identifier,
    /// Table name.
    TableName,
    /// Column name.
    ColumnName,
    /// JOIN clause.
    JoinClause,
    /// GROUP BY clause.
    GroupByClause,
    /// HAVING clause.
    HavingClause,
    /// ORDER BY clause.
    OrderByClause,
    /// LIMIT clause.
    LimitClause,
    /// Select item.
    SelectItem,
    /// Alias.
    Alias,
    /// Column definition.
    ColumnDefinition,
    /// Value list.
    ValueList,
    /// Assignment.
    Assignment,
    /// Alter action.
    AlterAction,
    /// Error node.
    ErrorNode,

    /// Whitespace.
    Whitespace,
    /// Newline.
    Newline,

    /// Comment.
    Comment,
    /// Line comment.
    LineComment,
    /// Block comment.
    BlockComment,

    /// Number literal.
    NumberLiteral,
    /// Float literal.
    FloatLiteral,
    /// String literal.
    StringLiteral,
    /// Boolean literal.
    BooleanLiteral,
    /// NULL literal.
    NullLiteral,

    /// Identifier token.
    Identifier_,

    /// `SELECT` keyword.
    Select,
    /// `FROM` keyword.
    From,
    /// `WHERE` keyword.
    Where,
    /// `INSERT` keyword.
    Insert,
    /// `INTO` keyword.
    Into,
    /// `VALUES` keyword.
    Values,
    /// `UPDATE` keyword.
    Update,
    /// `SET` keyword.
    Set,
    /// `DELETE` keyword.
    Delete,
    /// `CREATE` keyword.
    Create,
    /// `TABLE` keyword.
    Table,
    /// `DROP` keyword.
    Drop,
    /// `ALTER` keyword.
    Alter,
    /// `ADD` keyword.
    Add,
    /// `COLUMN` keyword.
    Column,
    /// `PRIMARY` keyword.
    Primary,
    /// `KEY` keyword.
    Key,
    /// `FOREIGN` keyword.
    Foreign,
    /// `REFERENCES` keyword.
    References,
    /// `INDEX` keyword.
    Index,
    /// `UNIQUE` keyword.
    Unique,
    /// `NOT` keyword.
    Not,
    /// `NULL` keyword.
    Null,
    /// `DEFAULT` keyword.
    Default,
    /// `AUTO_INCREMENT` keyword.
    AutoIncrement,
    /// `AND` keyword.
    And,
    /// `OR` keyword.
    Or,
    /// `IN` keyword.
    In,
    /// `LIKE` keyword.
    Like,
    /// `BETWEEN` keyword.
    Between,
    /// `IS` keyword.
    Is,
    /// `AS` keyword.
    As,
    /// `JOIN` keyword.
    Join,
    /// `INNER` keyword.
    Inner,
    /// `LEFT` keyword.
    Left,
    /// `RIGHT` keyword.
    Right,
    /// `FULL` keyword.
    Full,
    /// `OUTER` keyword.
    Outer,
    /// `ON` keyword.
    On,
    /// `GROUP` keyword.
    Group,
    /// `BY` keyword.
    By,
    /// `HAVING` keyword.
    Having,
    /// `ORDER` keyword.
    Order,
    /// `ASC` keyword.
    Asc,
    /// `DESC` keyword.
    Desc,
    /// `LIMIT` keyword.
    Limit,
    /// `OFFSET` keyword.
    Offset,
    /// `UNION` keyword.
    Union,
    /// `ALL` keyword.
    All,
    /// `DISTINCT` keyword.
    Distinct,
    /// `COUNT` keyword.
    Count,
    /// `SUM` keyword.
    Sum,
    /// `AVG` keyword.
    Avg,
    /// `MIN` keyword.
    Min,
    /// `EXPLAIN` keyword.
    Explain,
    /// `PRAGMA` keyword.
    Pragma,
    /// `SHOW` keyword.
    Show,
    /// `VIEW` keyword.
    View,
    /// `DATABASE` keyword.
    Database,
    /// `SCHEMA` keyword.
    Schema,
    /// `TRUE` keyword.
    True,
    /// `FALSE` keyword.
    False,
    /// `EXISTS` keyword.
    Exists,
    /// `TRIGGER` keyword.
    Trigger,
    /// `AFTER` keyword.
    After,
    /// `DELIMITER` keyword.
    Delimiter,
    /// `FOR` keyword.
    For,
    /// `EACH` keyword.
    Each,
    /// `ROW` keyword.
    Row,
    /// `CHECK` keyword.
    Check,
    /// `RENAME` keyword.
    Rename,
    /// `TO` keyword.
    To,
    /// `CASE` keyword.
    Case,
    /// `WHEN` keyword.
    When,
    /// `THEN` keyword.
    Then,
    /// `ELSE` keyword.
    Else,
    /// `END` keyword.
    End,
    /// `IF` keyword.
    If,
    /// `BEGIN` keyword.
    Begin,
    /// `COMMIT` keyword.
    Commit,
    /// `ROLLBACK` keyword.
    Rollback,
    /// `TRANSACTION` keyword.
    Transaction,
    /// `CONFLICT` keyword.
    Conflict,
    /// `DO` keyword.
    Do,
    /// `NOTHING` keyword.
    Nothing,
    /// `RETURNING` keyword.
    Returning,
    /// `VECTOR` keyword.
    Vector,
    /// `ILIKE` keyword.
    Ilike,
    /// `STRICT` keyword.
    Strict,
    /// `WITHOUT` keyword.
    Without,
    /// `ROWID` keyword.
    Rowid,

    /// `INT` type.
    Int,
    /// `INTEGER` type.
    Integer,
    /// `VARCHAR` type.
    Varchar,
    /// `CHAR` type.
    Char,
    /// `TEXT` type.
    Text,
    /// `DATE` type.
    Date,
    /// `TIME` type.
    Time,
    /// `TIMESTAMP` type.
    Timestamp,
    /// `DECIMAL` type.
    Decimal,
    /// `FLOAT` type.
    Float,
    /// `DOUBLE` type.
    Double,
    /// `BOOLEAN` type.
    Boolean,
    /// `SERIAL` type.
    Serial,
    /// `BIGSERIAL` type.
    BigSerial,

    /// Plus operator `+`.
    Plus,
    /// Minus operator `-`.
    Minus,
    /// Multiplication operator `*`.
    Star,
    /// Division operator `/`.
    Slash,
    /// Modulo operator `%`.
    Percent,
    /// Equality operator `=`.
    Equal,
    /// Inequality operator `<>`.
    NotEqual,
    /// Less than operator `<`.
    Less,
    /// Greater than operator `>`.
    Greater,
    /// Less than or equal operator `<=`.
    LessEqual,
    /// Greater than or equal operator `>=`.
    GreaterEqual,
    /// Concatenation operator `||`.
    Concat,
    /// Double Colon `::`.
    DoubleColon,
    /// Left parenthesis `(`.
    LeftParen,
    /// Right parenthesis `)`.
    RightParen,
    /// Left bracket `[`.
    LeftBracket,
    /// Right bracket `]`.
    RightBracket,
    /// Left brace `{`.
    LeftBrace,
    /// Right brace `}`.
    RightBrace,
    /// Comma `,`.
    Comma,
    /// Semicolon `;`.
    Semicolon,
    /// Dot `.`.
    Dot,
    /// Colon `:`.
    Colon,
    /// Question mark `?`.
    Question,

    /// Error token.
    Error,
    /// End of stream.
    Eof,
}

impl From<crate::parser::element_type::SqlElementType> for SqlTokenType {
    fn from(element: crate::parser::element_type::SqlElementType) -> Self {
        use crate::parser::element_type::SqlElementType;
        match element {
            SqlElementType::Root => Self::Root,
            SqlElementType::ExplainStatement => Self::ExplainStatement,
            SqlElementType::Identifier => Self::Identifier,
            SqlElementType::Expression => Self::Expression,
            SqlElementType::ErrorNode => Self::ErrorNode,
            SqlElementType::SelectStatement => Self::SelectStatement,
            SqlElementType::VectorSearch => Self::Vector, // Map VectorSearch to Vector token for now or add a new one?
            SqlElementType::InsertStatement => Self::InsertStatement,
            SqlElementType::UpdateStatement => Self::UpdateStatement,
            SqlElementType::DeleteStatement => Self::DeleteStatement,
            SqlElementType::CreateStatement => Self::CreateStatement,
            SqlElementType::DropStatement => Self::DropStatement,
            SqlElementType::AlterStatement => Self::AlterStatement,
            SqlElementType::JoinClause => Self::JoinClause,
            SqlElementType::GroupByClause => Self::GroupByClause,
            SqlElementType::HavingClause => Self::HavingClause,
            SqlElementType::OrderByClause => Self::OrderByClause,
            SqlElementType::LimitClause => Self::LimitClause,
            SqlElementType::TableName => Self::TableName,
            SqlElementType::ColumnName => Self::ColumnName,
            SqlElementType::SelectItem => Self::SelectItem,
            SqlElementType::Alias => Self::Alias,
            SqlElementType::ColumnDefinition => Self::ColumnDefinition,
            SqlElementType::ValueList => Self::ValueList,
            SqlElementType::Assignment => Self::Assignment,
            SqlElementType::AlterAction => Self::AlterAction,
            _ => Self::ErrorNode,
        }
    }
}
