use crate::{Arc, source::DocumentSnapshot};

use super::Vfs;

/// Editor document lifecycle for unsaved buffers layered over file access.
pub trait DocumentSession: Vfs {
    /// Opens or replaces a document buffer.
    fn open_document(&self, uri: &str, text: Arc<str>) -> DocumentSnapshot;

    /// Applies a full-text change to an open document.
    fn change_document(&self, uri: &str, text: Arc<str>) -> DocumentSnapshot {
        self.open_document(uri, text)
    }

    /// Closes a document and drops its revision state.
    fn close_document(&self, uri: &str);

    /// Returns the latest snapshot for an open document.
    fn document_snapshot(&self, uri: &str) -> Option<DocumentSnapshot>;

    /// Returns source text for language requests, preferring an open document snapshot.
    fn request_source(&self, uri: &str) -> Option<Self::Source> {
        self.get_source(uri)
    }
}
