use std::{
    collections::HashMap,
    sync::{
        RwLock,
        atomic::{AtomicU64, Ordering},
    },
};

use crate::{Arc, source::SnapshotRevision};

/// Tracks monotonic document revisions keyed by URI.
#[derive(Debug, Default)]
pub struct RevisionTracker {
    next: AtomicU64,
    by_uri: RwLock<HashMap<Arc<str>, SnapshotRevision>>,
}

impl RevisionTracker {
    /// Creates an empty revision tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Assigns a new revision to `uri` and returns it.
    pub fn bump(&self, uri: impl Into<Arc<str>>) -> SnapshotRevision {
        let revision = SnapshotRevision::new(self.next.fetch_add(1, Ordering::SeqCst));
        self.by_uri.write().expect("revision tracker lock poisoned").insert(uri.into(), revision);
        revision
    }

    /// Returns the latest revision for `uri`, if any.
    pub fn get(&self, uri: &str) -> Option<SnapshotRevision> {
        self.by_uri.read().expect("revision tracker lock poisoned").get(uri).copied()
    }

    /// Drops revision state for a closed document.
    pub fn clear(&self, uri: &str) {
        self.by_uri.write().expect("revision tracker lock poisoned").remove(uri);
    }
}
