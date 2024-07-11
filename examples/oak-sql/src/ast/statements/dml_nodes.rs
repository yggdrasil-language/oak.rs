use crate::ast::{
    expr::{Expression, Identifier, TableName},
    statements::query::{SelectItem, SelectStatement},
};
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};

/// Represents an INSERT statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InsertStatement {
    /// The table to insert into.
    pub table_name: TableName,
    /// The columns to insert into.
    pub columns: Vec<Identifier>,
    /// The rows of a `VALUES` list, one vector per row.
    ///
    /// Empty when the rows are produced by a nested query instead.
    pub rows: Vec<Vec<Expression>>,
    /// Where the inserted rows come from.
    pub source: InsertSource,
    /// The `ON CONFLICT` clause, when one was written.
    pub on_conflict: Option<OnConflictClause>,
    /// The `RETURNING` clause, when one was written.
    pub returning: Option<ReturningClause>,
    /// The span of the INSERT statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for InsertStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("INSERT");
        buffer.push("INTO");
        self.table_name.to_source(buffer);
        if !self.columns.is_empty() {
            buffer.push("(");
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    buffer.push(",");
                }
                col.to_source(buffer);
            }
            buffer.push(")");
        }
        match &self.source {
            InsertSource::Values => {
                buffer.push("VALUES");
                for (i, row) in self.rows.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    buffer.push("(");
                    for (j, val) in row.iter().enumerate() {
                        if j > 0 {
                            buffer.push(",");
                        }
                        val.to_source(buffer);
                    }
                    buffer.push(")");
                }
            }
            InsertSource::Select(query) => query.to_source(buffer),
        }
        if let Some(on_conflict) = &self.on_conflict {
            on_conflict.to_source(buffer);
        }
        if let Some(returning) = &self.returning {
            returning.to_source(buffer);
        }
    }
}

/// The origin of the rows inserted by an `INSERT` statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InsertSource {
    /// Rows written out in a `VALUES` list.
    Values,
    /// Rows produced by a nested query.
    Select(Box<SelectStatement>),
}

/// Represents an `ON CONFLICT` clause of an `INSERT` statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OnConflictClause {
    /// The conflicting columns the clause applies to.
    pub columns: Vec<Identifier>,
    /// The action taken when a conflict happens.
    pub action: OnConflictAction,
    /// The span of the clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for OnConflictClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("ON");
        buffer.push("CONFLICT");
        if !self.columns.is_empty() {
            buffer.push("(");
            for (i, col) in self.columns.iter().enumerate() {
                if i > 0 {
                    buffer.push(",");
                }
                col.to_source(buffer);
            }
            buffer.push(")");
        }
        self.action.to_source(buffer);
    }
}

/// The action of an `ON CONFLICT` clause.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OnConflictAction {
    /// `DO NOTHING`.
    DoNothing,
    /// `DO UPDATE SET ... [WHERE ...]`.
    DoUpdate {
        /// The assignments applied to the conflicting row.
        assignments: Vec<Assignment>,
        /// The predicate restricting the rows the update applies to.
        selection: Option<Expression>,
    },
}

impl ToSource for OnConflictAction {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("DO");
        match self {
            OnConflictAction::DoNothing => buffer.push("NOTHING"),
            OnConflictAction::DoUpdate { assignments, selection } => {
                buffer.push("UPDATE");
                buffer.push("SET");
                for (i, assignment) in assignments.iter().enumerate() {
                    if i > 0 {
                        buffer.push(",");
                    }
                    assignment.to_source(buffer);
                }
                if let Some(selection) = selection {
                    buffer.push("WHERE");
                    selection.to_source(buffer);
                }
            }
        }
    }
}

/// Represents a `RETURNING` clause of an `INSERT` statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReturningClause {
    /// The returned items.
    pub items: Vec<SelectItem>,
    /// The span of the clause.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for ReturningClause {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("RETURNING");
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                buffer.push(",");
            }
            item.to_source(buffer);
        }
    }
}

/// Represents an UPDATE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UpdateStatement {
    /// The table to update.
    pub table_name: TableName,
    /// The assignments to perform.
    pub assignments: Vec<Assignment>,
    /// The WHERE clause selection expression.
    pub selection: Option<Expression>, // WHERE clause
    /// The span of the UPDATE statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for UpdateStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("UPDATE");
        self.table_name.to_source(buffer);
        buffer.push("SET");
        for (i, assignment) in self.assignments.iter().enumerate() {
            if i > 0 {
                buffer.push(",");
            }
            assignment.to_source(buffer);
        }
        if let Some(selection) = &self.selection {
            buffer.push("WHERE");
            selection.to_source(buffer);
        }
    }
}

/// Represents an assignment in an UPDATE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Assignment {
    /// The column to assign to.
    pub column: Identifier,
    /// The expression to assign.
    pub value: Expression,
    /// The span of the assignment.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for Assignment {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.column.to_source(buffer);
        buffer.push("=");
        self.value.to_source(buffer);
    }
}

/// Represents a DELETE statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DeleteStatement {
    /// The table to delete from.
    pub table_name: TableName,
    /// The WHERE clause selection expression.
    pub selection: Option<Expression>, // WHERE clause
    /// The span of the DELETE statement.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for DeleteStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("DELETE");
        buffer.push("FROM");
        self.table_name.to_source(buffer);
        if let Some(selection) = &self.selection {
            buffer.push("WHERE");
            selection.to_source(buffer);
        }
    }
}
