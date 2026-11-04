use crate::ast::{
    expr::{Expression, Identifier, TableName},
    statements::query::{OrderDirection, SelectStatement},
};
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
use std::sync::Arc;

/// Represents a CREATE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CreateStatement {
    /// The type of object to create (TABLE, VIEW, etc.).
    pub object_type: CreateObjectType,
    /// The name of the object.
    pub name: Identifier,
    /// Whether to include IF NOT EXISTS.
    pub if_not_exists: bool,
    /// The body of the CREATE statement.
    pub body: CreateBody,
    /// The span of the CREATE statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

/// Represents the body of a CREATE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateBody {
    /// A TABLE body with column definitions and table-level constraints.
    Table {
        /// The column definitions.
        columns: Vec<ColumnDefinition>,
        /// The table-level constraints, in source order.
        constraints: Vec<TableConstraint>,
        /// The span of the TABLE body.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// A VIEW body with a query.
    View {
        /// The query for the view.
        query: Box<SelectStatement>,
        /// The span of the VIEW body.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// An INDEX body.
    Index {
        /// The table to create the index on.
        table_name: TableName,
        /// The columns to index.
        columns: Vec<Identifier>,
        /// Whether it's a UNIQUE index.
        unique: bool,
        /// The span of the INDEX body.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// A DATABASE body.
    Database {
        /// The span of the DATABASE body.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for CreateStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("CREATE");
        if let CreateBody::Index { unique: true, .. } = &self.body {
            buffer.push("UNIQUE");
        }
        self.object_type.to_source(buffer);
        if self.if_not_exists {
            buffer.push("IF");
            buffer.push("NOT");
            buffer.push("EXISTS");
        }
        self.name.to_source(buffer);
        match &self.body {
            CreateBody::Table { columns, constraints, .. } => {
                if !columns.is_empty() || !constraints.is_empty() {
                    buffer.push("(");
                    for (i, col) in columns.iter().enumerate() {
                        if i > 0 {
                            buffer.push(",");
                        }
                        col.to_source(buffer);
                    }
                    for (i, constraint) in constraints.iter().enumerate() {
                        if i > 0 || !columns.is_empty() {
                            buffer.push(",");
                        }
                        constraint.to_source(buffer);
                    }
                    buffer.push(")");
                }
            }
            CreateBody::View { query, .. } => {
                buffer.push("AS");
                query.to_source(buffer);
            }
            CreateBody::Index { table_name, columns, .. } => {
                buffer.push("ON");
                table_name.to_source(buffer);
                buffer.push("(");
                for (i, col) in columns.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    col.to_source(buffer);
                }
                buffer.push(")");
            }
            CreateBody::Database { .. } => {}
        }
    }
}

/// Represents a column definition in a CREATE TABLE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColumnDefinition {
    /// The name of the column.
    pub name: Identifier,
    /// The data type of the column.
    pub data_type: Arc<str>,
    /// Any column constraints.
    pub constraints: Vec<ColumnConstraint>,
    /// The span of the column definition.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for ColumnDefinition {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
        buffer.push(&self.data_type);
        for constraint in &self.constraints {
            constraint.to_source(buffer);
        }
    }
}

/// Represents a constraint on a column.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ColumnConstraint {
    /// PRIMARY KEY constraint.
    PrimaryKey {
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// NOT NULL constraint.
    NotNull {
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// NULL constraint.
    Nullable {
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// UNIQUE constraint.
    Unique {
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// DEFAULT constraint with an expression.
    Default(Expression, #[serde(with = "oak_core::serde_range")] Range<usize>),
    /// CHECK constraint with an expression.
    Check(Expression, #[serde(with = "oak_core::serde_range")] Range<usize>),
    /// AUTOINCREMENT constraint.
    AutoIncrement {
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for ColumnConstraint {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            ColumnConstraint::PrimaryKey { .. } => {
                buffer.push("PRIMARY");
                buffer.push("KEY");
            }
            ColumnConstraint::NotNull { .. } => {
                buffer.push("NOT");
                buffer.push("NULL");
            }
            ColumnConstraint::Nullable { .. } => buffer.push("NULL"),
            ColumnConstraint::Unique { .. } => buffer.push("UNIQUE"),
            ColumnConstraint::Default(expr, _) => {
                buffer.push("DEFAULT");
                expr.to_source(buffer);
            }
            ColumnConstraint::Check(expr, _) => {
                buffer.push("CHECK");
                buffer.push("(");
                expr.to_source(buffer);
                buffer.push(")");
            }
            ColumnConstraint::AutoIncrement { .. } => buffer.push("AUTOINCREMENT"),
        }
    }
}

/// Represents a constraint declared for the whole table instead of a single
/// column.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TableConstraint {
    /// `PRIMARY KEY (col [ASC|DESC], ...) [ON CONFLICT resolution]`
    PrimaryKey {
        /// The indexed columns, in source order.
        columns: Vec<IndexedColumn>,
        /// The `ON CONFLICT` resolution, when one was written.
        on_conflict: Option<ConflictResolution>,
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// `FOREIGN KEY (col, ...) REFERENCES table [(col, ...)]`
    ForeignKey {
        /// The constrained columns of the current table.
        columns: Vec<Identifier>,
        /// The referenced table.
        table_name: TableName,
        /// The referenced columns, empty when the reference names no columns.
        referenced_columns: Vec<Identifier>,
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// `CHECK (expr)`
    Check {
        /// The checked expression.
        expr: Expression,
        /// Source span of the constraint.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for TableConstraint {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            TableConstraint::PrimaryKey { columns, on_conflict, .. } => {
                buffer.push("PRIMARY");
                buffer.push("KEY");
                buffer.push("(");
                for (i, column) in columns.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    column.to_source(buffer);
                }
                buffer.push(")");
                if let Some(resolution) = on_conflict {
                    buffer.push(" ON");
                    buffer.push("CONFLICT");
                    resolution.to_source(buffer);
                }
            }
            TableConstraint::ForeignKey { columns, table_name, referenced_columns, .. } => {
                buffer.push("FOREIGN");
                buffer.push("KEY");
                buffer.push("(");
                for (i, column) in columns.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    column.to_source(buffer);
                }
                buffer.push(")");
                buffer.push(" REFERENCES");
                table_name.to_source(buffer);
                if !referenced_columns.is_empty() {
                    buffer.push("(");
                    for (i, column) in referenced_columns.iter().enumerate() {
                        if i > 0 {
                            buffer.push(",");
                        }
                        column.to_source(buffer);
                    }
                    buffer.push(")");
                }
            }
            TableConstraint::Check { expr, .. } => {
                buffer.push("CHECK");
                buffer.push("(");
                expr.to_source(buffer);
                buffer.push(")");
            }
        }
    }
}

/// A column of a table-level `PRIMARY KEY`, with its optional ordering.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IndexedColumn {
    /// The column name.
    pub name: Identifier,
    /// The optional `ASC` / `DESC` ordering.
    pub direction: Option<OrderDirection>,
    /// Source span of the indexed column.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for IndexedColumn {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.name.to_source(buffer);
        if let Some(direction) = &self.direction {
            direction.to_source(buffer);
        }
    }
}

/// The conflict resolution of an `ON CONFLICT` clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConflictResolution {
    /// `ON CONFLICT ROLLBACK`
    Rollback,
    /// `ON CONFLICT ABORT`
    Abort,
    /// `ON CONFLICT FAIL`
    Fail,
    /// `ON CONFLICT IGNORE`
    Ignore,
    /// `ON CONFLICT REPLACE`
    Replace,
}

impl ConflictResolution {
    /// Returns the resolution written as `word`, or `None` when it is unknown.
    pub fn from_word(word: &str) -> Option<Self> {
        match word.to_ascii_uppercase().as_str() {
            "ROLLBACK" => Some(Self::Rollback),
            "ABORT" => Some(Self::Abort),
            "FAIL" => Some(Self::Fail),
            "IGNORE" => Some(Self::Ignore),
            "REPLACE" => Some(Self::Replace),
            _ => None,
        }
    }
}

impl ToSource for ConflictResolution {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            ConflictResolution::Rollback => buffer.push("ROLLBACK"),
            ConflictResolution::Abort => buffer.push("ABORT"),
            ConflictResolution::Fail => buffer.push("FAIL"),
            ConflictResolution::Ignore => buffer.push("IGNORE"),
            ConflictResolution::Replace => buffer.push("REPLACE"),
        }
    }
}

/// Represents the type of object to create.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CreateObjectType {
    /// TABLE object.
    Table,
    /// VIEW object.
    View,
    /// INDEX object.
    Index,
    /// DATABASE object.
    Database,
}

impl ToSource for CreateObjectType {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            CreateObjectType::Table => buffer.push("TABLE"),
            CreateObjectType::View => buffer.push("VIEW"),
            CreateObjectType::Index => buffer.push("INDEX"),
            CreateObjectType::Database => buffer.push("DATABASE"),
        }
    }
}

/// Represents a DROP statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DropStatement {
    /// The type of object to drop.
    pub object_type: DropObjectType,
    /// The name of the object.
    pub name: Identifier,
    /// Whether to include IF EXISTS.
    pub if_exists: bool,
    /// The span of the DROP statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for DropStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("DROP");
        self.object_type.to_source(buffer);
        if self.if_exists {
            buffer.push("IF");
            buffer.push("EXISTS");
        }
        self.name.to_source(buffer);
    }
}

/// Represents the type of object to drop.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DropObjectType {
    /// TABLE object.
    Table,
    /// VIEW object.
    View,
    /// INDEX object.
    Index,
    /// DATABASE object.
    Database,
}

impl ToSource for DropObjectType {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            DropObjectType::Table => buffer.push("TABLE"),
            DropObjectType::View => buffer.push("VIEW"),
            DropObjectType::Index => buffer.push("INDEX"),
            DropObjectType::Database => buffer.push("DATABASE"),
        }
    }
}

/// Represents an ALTER statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AlterStatement {
    /// The table to alter.
    pub table_name: TableName,
    /// The action to perform.
    pub action: Option<AlterAction>,
    /// The span of the ALTER statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

/// Represents an action in an ALTER TABLE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AlterAction {
    /// ADD COLUMN action.
    AddColumn {
        /// The name of the column to add.
        name: Identifier,
        /// The data type of the column.
        data_type: Option<Arc<str>>,
        /// The span of the ADD COLUMN action.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// DROP COLUMN action.
    DropColumn {
        /// The name of the column to drop.
        name: Identifier,
        /// The span of the DROP COLUMN action.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
    /// RENAME TO action.
    RenameTo {
        /// The new name for the table.
        new_name: Identifier,
        /// The span of the RENAME TO action.
        #[serde(with = "oak_core::serde_range")]
        span: Range<usize>,
    },
}

impl ToSource for AlterStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("ALTER");
        buffer.push("TABLE");
        self.table_name.to_source(buffer);
        if let Some(action) = &self.action {
            match action {
                AlterAction::AddColumn { name, data_type, .. } => {
                    buffer.push("ADD");
                    buffer.push("COLUMN");
                    name.to_source(buffer);
                    if let Some(dt) = data_type {
                        buffer.push(dt);
                    }
                }
                AlterAction::DropColumn { name, .. } => {
                    buffer.push("DROP");
                    buffer.push("COLUMN");
                    name.to_source(buffer);
                }
                AlterAction::RenameTo { new_name, .. } => {
                    buffer.push("RENAME");
                    buffer.push("TO");
                    new_name.to_source(buffer);
                }
            }
        }
    }
}
