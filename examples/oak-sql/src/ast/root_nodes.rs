use crate::ast::statements::{
    ddl::{AlterStatement, CreateStatement, DropStatement},
    dml::{DeleteStatement, InsertStatement, UpdateStatement},
    query::SelectStatement,
    session::{SetNamesStatement, TransactionStatement},
};
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
use std::sync::Arc;

/// SQL root node representing a collection of SQL statements.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SqlRoot {
    /// The list of SQL statements in the script.
    pub statements: Vec<SqlStatement>,
    /// The full span of the SQL script.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for SqlRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        for (i, stmt) in self.statements.iter().enumerate() {
            if i > 0 {
                buffer.push(";");
            }
            stmt.to_source(buffer);
        }
        if !self.statements.is_empty() {
            buffer.push(";");
        }
    }
}

/// Represents a single SQL statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SqlStatement {
    /// A SELECT statement.
    Select(SelectStatement),
    /// An INSERT statement.
    Insert(InsertStatement),
    /// An UPDATE statement.
    Update(UpdateStatement),
    /// A DELETE statement.
    Delete(DeleteStatement),
    /// A CREATE statement.
    Create(CreateStatement),
    /// A DROP statement.
    Drop(DropStatement),
    /// An ALTER statement.
    Alter(AlterStatement),
    /// A transaction boundary.
    Transaction(TransactionStatement),
    /// A character-set session setting.
    SetNames(SetNamesStatement),
    /// An error occurred during parsing or building.
    Error {
        /// The error message.
        message: Arc<str>,
        /// The span where the error occurred.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// An unknown or unparsed statement.
    Unknown {
        /// The span of the unknown statement.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for SqlStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            SqlStatement::Select(s) => s.to_source(buffer),
            SqlStatement::Insert(s) => s.to_source(buffer),
            SqlStatement::Update(s) => s.to_source(buffer),
            SqlStatement::Delete(s) => s.to_source(buffer),
            SqlStatement::Create(s) => s.to_source(buffer),
            SqlStatement::Drop(s) => s.to_source(buffer),
            SqlStatement::Alter(s) => s.to_source(buffer),
            SqlStatement::Transaction(s) => s.to_source(buffer),
            SqlStatement::SetNames(s) => s.to_source(buffer),
            SqlStatement::Error { message, .. } => {
                buffer.push("/* ERROR: ");
                buffer.push(message);
                buffer.push(" */");
            }
            SqlStatement::Unknown { .. } => {}
        }
    }
}
