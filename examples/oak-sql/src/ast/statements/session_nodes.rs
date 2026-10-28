use super::super::expression_nodes::Expression;
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
use std::sync::Arc;

/// Transaction boundary operation.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionAction {
    /// Start a transaction.
    Begin,
    /// Commit the active transaction.
    Commit,
    /// Roll back the active transaction.
    Rollback,
    /// Create a named savepoint.
    Savepoint(Arc<str>),
    /// Roll back to a named savepoint (`ROLLBACK TO SAVEPOINT name` / `ROLLBACK TO name`).
    RollbackToSavepoint(Arc<str>),
    /// Release (commit) a named savepoint (`RELEASE SAVEPOINT name` / `RELEASE name`).
    ReleaseSavepoint(Arc<str>),
}

/// A transaction boundary statement.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TransactionStatement {
    /// Requested transaction operation.
    pub action: TransactionAction,
    /// Source span.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for TransactionStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match &self.action {
            TransactionAction::Begin => buffer.push("BEGIN"),
            TransactionAction::Commit => buffer.push("COMMIT"),
            TransactionAction::Rollback => buffer.push("ROLLBACK"),
            TransactionAction::Savepoint(name) => {
                buffer.push("SAVEPOINT ");
                buffer.push(name);
            }
            TransactionAction::RollbackToSavepoint(name) => {
                buffer.push("ROLLBACK TO SAVEPOINT ");
                buffer.push(name);
            }
            TransactionAction::ReleaseSavepoint(name) => {
                buffer.push("RELEASE SAVEPOINT ");
                buffer.push(name);
            }
        }
    }
}

/// A MySQL-compatible character-set session statement.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetNamesStatement {
    /// Requested connection character set.
    pub character_set: Arc<str>,
    /// Optional requested connection collation.
    pub collation: Option<Arc<str>>,
    /// Source span.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for SetNamesStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("SET NAMES ");
        buffer.push(&self.character_set);
        if let Some(collation) = &self.collation {
            buffer.push(" COLLATE ");
            buffer.push(collation);
        }
    }
}

/// Scope prefix carried by `SET` and `SHOW` session statements.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SessionScope {
    /// `GLOBAL` — server-wide variable scope.
    Global,
    /// `SESSION` — connection-scoped variable scope.
    Session,
    /// `LOCAL` — dialect-specific local scope.
    Local,
}

impl SessionScope {
    /// Parses a scope keyword, case-insensitively.
    pub fn from_word(word: &str) -> Option<Self> {
        match word.to_ascii_uppercase().as_str() {
            "GLOBAL" => Some(Self::Global),
            "SESSION" => Some(Self::Session),
            "LOCAL" => Some(Self::Local),
            _ => None,
        }
    }

    /// Keyword spelling.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Global => "GLOBAL",
            Self::Session => "SESSION",
            Self::Local => "LOCAL",
        }
    }
}

/// A `SET <scope?> <name> = <value>` session variable statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SetVariableStatement {
    /// Optional `GLOBAL` / `SESSION` / `LOCAL` prefix.
    pub scope: Option<SessionScope>,
    /// Variable name as written by the client.
    pub name: Arc<str>,
    /// Assigned value expression.
    pub value: Expression,
    /// Source span.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for SetVariableStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("SET ");
        if let Some(scope) = self.scope {
            buffer.push(scope.as_str());
            buffer.push(" ");
        }
        buffer.push(&self.name);
        buffer.push(" = ");
        self.value.to_source(buffer);
    }
}

/// A `SHOW <scope?> <target> [LIKE '<pattern>']` statement.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ShowStatement {
    /// Optional `GLOBAL` / `SESSION` prefix.
    pub scope: Option<SessionScope>,
    /// Target word: `VARIABLES`, `STATUS`, or a single parameter name.
    pub target: Arc<str>,
    /// Optional `LIKE` pattern, with quotes removed.
    pub like: Option<Arc<str>>,
    /// Source span.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for ShowStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("SHOW ");
        if let Some(scope) = self.scope {
            buffer.push(scope.as_str());
            buffer.push(" ");
        }
        buffer.push(&self.target);
        if let Some(pattern) = &self.like {
            buffer.push(" LIKE '");
            buffer.push(pattern);
            buffer.push("'");
        }
    }
}

/// A SQLite `PRAGMA <name> [= <value>]` statement.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PragmaStatement {
    /// Pragma name as written by the client.
    pub name: Arc<str>,
    /// Optional assigned value.
    pub value: Option<Expression>,
    /// Source span.
    #[serde(with = "oak_core::serde_range")]
    pub span: Range<usize>,
}

impl ToSource for PragmaStatement {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("PRAGMA ");
        buffer.push(&self.name);
        if let Some(value) = &self.value {
            buffer.push(" = ");
            value.to_source(buffer);
        }
    }
}
