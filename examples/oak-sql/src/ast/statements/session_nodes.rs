use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
use std::sync::Arc;

/// Transaction boundary operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TransactionAction {
    /// Start a transaction.
    Begin,
    /// Commit the active transaction.
    Commit,
    /// Roll back the active transaction.
    Rollback,
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
        buffer.push(match self.action {
            TransactionAction::Begin => "BEGIN",
            TransactionAction::Commit => "COMMIT",
            TransactionAction::Rollback => "ROLLBACK",
        });
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
