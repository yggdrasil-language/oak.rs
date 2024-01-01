use oak_core::{ElementType, UniversalElementRole};

/// Element types for the SQL language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SqlElementType {
    /// Root node.
    Root,
    /// EXPLAIN statement.
    ExplainStatement,
    /// TRANSACTION statement.
    TransactionStatement,
    /// `SET NAMES` session statement.
    SetNamesStatement,
    /// PRAGMA statement.
    PragmaStatement,
    /// SHOW statement.
    ShowStatement,
    /// SELECT statement.
    SelectStatement,
    /// VECTOR search clause.
    VectorSearch,
    /// Identifier.
    Identifier,
    /// Expression.
    Expression,
    /// Error node.
    ErrorNode,
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
    /// Table name.
    TableName,
    /// Column name.
    ColumnName,
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
    /// RETURNING clause.
    ReturningClause,
    /// ON CONFLICT clause.
    ConflictClause,
    /// Subquery.
    Subquery,
    /// Function call.
    FunctionCall,
}

impl ElementType for SqlElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> UniversalElementRole {
        use UniversalElementRole::*;
        match self {
            Self::Root => Root,
            Self::Identifier | Self::TableName | Self::ColumnName | Self::Alias => Name,
            Self::Expression | Self::FunctionCall => Expression,
            Self::ErrorNode => Error,
            Self::SelectStatement
            | Self::InsertStatement
            | Self::UpdateStatement
            | Self::DeleteStatement
            | Self::CreateStatement
            | Self::DropStatement
            | Self::AlterStatement
            | Self::ExplainStatement
            | Self::TransactionStatement
            | Self::SetNamesStatement
            | Self::PragmaStatement
            | Self::ShowStatement => Statement,
            Self::JoinClause
            | Self::GroupByClause
            | Self::HavingClause
            | Self::OrderByClause
            | Self::LimitClause
            | Self::SelectItem
            | Self::ColumnDefinition
            | Self::ValueList
            | Self::Assignment
            | Self::AlterAction
            | Self::ReturningClause
            | Self::ConflictClause
            | Self::VectorSearch
            | Self::Subquery => Statement,
        }
    }
}

impl From<crate::lexer::token_type::SqlTokenType> for SqlElementType {
    fn from(token: crate::lexer::token_type::SqlTokenType) -> Self {
        match token {
            crate::lexer::token_type::SqlTokenType::Root => Self::Root,
            crate::lexer::token_type::SqlTokenType::ExplainStatement => Self::ExplainStatement,
            crate::lexer::token_type::SqlTokenType::TransactionStatement => Self::TransactionStatement,
            crate::lexer::token_type::SqlTokenType::PragmaStatement => Self::PragmaStatement,
            crate::lexer::token_type::SqlTokenType::ShowStatement => Self::ShowStatement,
            crate::lexer::token_type::SqlTokenType::SelectStatement => Self::SelectStatement,
            crate::lexer::token_type::SqlTokenType::InsertStatement => Self::InsertStatement,
            crate::lexer::token_type::SqlTokenType::UpdateStatement => Self::UpdateStatement,
            crate::lexer::token_type::SqlTokenType::DeleteStatement => Self::DeleteStatement,
            crate::lexer::token_type::SqlTokenType::CreateStatement => Self::CreateStatement,
            crate::lexer::token_type::SqlTokenType::DropStatement => Self::DropStatement,
            crate::lexer::token_type::SqlTokenType::AlterStatement => Self::AlterStatement,
            crate::lexer::token_type::SqlTokenType::Expression => Self::Expression,
            crate::lexer::token_type::SqlTokenType::Identifier => Self::Identifier,
            crate::lexer::token_type::SqlTokenType::TableName => Self::TableName,
            crate::lexer::token_type::SqlTokenType::ColumnName => Self::ColumnName,
            crate::lexer::token_type::SqlTokenType::JoinClause => Self::JoinClause,
            crate::lexer::token_type::SqlTokenType::GroupByClause => Self::GroupByClause,
            crate::lexer::token_type::SqlTokenType::HavingClause => Self::HavingClause,
            crate::lexer::token_type::SqlTokenType::OrderByClause => Self::OrderByClause,
            crate::lexer::token_type::SqlTokenType::LimitClause => Self::LimitClause,
            crate::lexer::token_type::SqlTokenType::SelectItem => Self::SelectItem,
            crate::lexer::token_type::SqlTokenType::Alias => Self::Alias,
            crate::lexer::token_type::SqlTokenType::ColumnDefinition => Self::ColumnDefinition,
            crate::lexer::token_type::SqlTokenType::ValueList => Self::ValueList,
            crate::lexer::token_type::SqlTokenType::Assignment => Self::Assignment,
            crate::lexer::token_type::SqlTokenType::AlterAction => Self::AlterAction,
            crate::lexer::token_type::SqlTokenType::ErrorNode => Self::ErrorNode,
            _ => Self::ErrorNode,
        }
    }
}
