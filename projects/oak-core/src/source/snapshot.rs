use super::SourceText;

/// Monotonic document revision assigned when editor content changes.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SnapshotRevision(u64);

impl SnapshotRevision {
    /// Creates a revision from its wire value.
    pub fn new(value: u64) -> Self {
        Self(value)
    }

    /// Returns the raw revision value.
    pub fn as_u64(self) -> u64 {
        self.0
    }
}

/// Immutable source text paired with the revision that produced it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DocumentSnapshot {
    source: SourceText,
    revision: SnapshotRevision,
}

impl DocumentSnapshot {
    /// Builds a snapshot from source text and revision.
    pub fn new(source: SourceText, revision: SnapshotRevision) -> Self {
        Self { source, revision }
    }

    /// Returns the source text for this snapshot.
    pub fn source(&self) -> &SourceText {
        &self.source
    }

    /// Returns the revision for this snapshot.
    pub fn revision(&self) -> SnapshotRevision {
        self.revision
    }
}
