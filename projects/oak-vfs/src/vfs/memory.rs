use std::path::Path;

use crate::{DocumentSession, FileMetadata, FileType, RevisionTracker, Vfs, WritableVfs};
use boost_fs::vfs::Overlay;
use oak_core::{
    Arc,
    source::{DocumentSnapshot, SourceId, SourceText},
};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{
        RwLock,
        atomic::{AtomicU32, Ordering},
    },
};

/// A memory-based Virtual File System backed by [`boost_fs::vfs::Overlay`].
#[derive(Default)]
pub struct MemoryVfs {
    overlay: Overlay,
    revisions: RevisionTracker,
    ids: Arc<RwLock<HashMap<SourceId, Arc<str>>>>,
    uri_to_id: Arc<RwLock<HashMap<Arc<str>, SourceId>>>,
    modified: Arc<RwLock<HashMap<Arc<str>, u64>>>,
    next_id: Arc<AtomicU32>,
}

impl Clone for MemoryVfs {
    fn clone(&self) -> Self {
        Self {
            overlay: Overlay::new(),
            revisions: RevisionTracker::new(),
            ids: Arc::new(RwLock::new(self.ids.read().unwrap().clone())),
            uri_to_id: Arc::new(RwLock::new(self.uri_to_id.read().unwrap().clone())),
            modified: Arc::new(RwLock::new(self.modified.read().unwrap().clone())),
            next_id: self.next_id.clone(),
        }
    }
}

impl MemoryVfs {
    /// Create a new empty MemoryVfs.
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns the revision tracker for open documents.
    pub fn revisions(&self) -> &RevisionTracker {
        &self.revisions
    }

    /// Upsert a file's content in the memory VFS.
    pub fn write_file(&self, uri: &str, content: impl Into<Arc<str>>) {
        let uri_arc: Arc<str> = Arc::from(uri);
        let content = content.into();
        self.overlay.write(PathBuf::from(uri), content.as_bytes().to_vec());
        self.revisions.bump(uri_arc.clone());
        self.modified.write().unwrap().insert(uri_arc.clone(), Self::now());

        let mut ids = self.ids.write().unwrap();
        let mut uri_to_id = self.uri_to_id.write().unwrap();
        if !uri_to_id.contains_key(&uri_arc) {
            let id = self.next_id.fetch_add(1, Ordering::SeqCst);
            uri_to_id.insert(uri_arc.clone(), id);
            ids.insert(id, uri_arc.clone());
        }
    }

    /// Remove a file from the memory VFS.
    pub fn remove_file(&self, uri: &str) {
        self.overlay.remove(Path::new(uri));
        self.revisions.clear(uri);
        let mut ids = self.ids.write().unwrap();
        let mut uri_to_id = self.uri_to_id.write().unwrap();
        let mut modified = self.modified.write().unwrap();
        if let Some(id) = uri_to_id.remove(uri) {
            ids.remove(&id);
        }
        modified.remove(uri);
    }

    fn snapshot_after_write(&self, uri: &str) -> DocumentSnapshot {
        let revision = self.revisions.get(uri).expect("revision was just assigned");
        let source = self.get_source(uri).expect("document was just written");
        DocumentSnapshot::new(source, revision)
    }

    fn now() -> u64 {
        #[cfg(not(target_arch = "wasm32"))]
        {
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
        }
        #[cfg(target_arch = "wasm32")]
        {
            0
        }
    }
}

impl DocumentSession for MemoryVfs {
    fn open_document(&self, uri: &str, text: Arc<str>) -> DocumentSnapshot {
        self.write_file(uri, text);
        self.snapshot_after_write(uri)
    }

    fn close_document(&self, uri: &str) {
        self.remove_file(uri);
    }

    fn document_snapshot(&self, uri: &str) -> Option<DocumentSnapshot> {
        let revision = self.revisions.get(uri)?;
        self.get_source(uri).map(|source| DocumentSnapshot::new(source, revision))
    }

    fn request_source(&self, uri: &str) -> Option<SourceText> {
        self.document_snapshot(uri)
            .map(|snapshot| snapshot.source().clone())
            .or_else(|| self.get_source(uri))
    }
}

impl Vfs for MemoryVfs {
    type Source = SourceText;

    fn get_source(&self, uri: &str) -> Option<SourceText> {
        let content = self.overlay.read(Path::new(uri))?;
        let id = self.uri_to_id.read().unwrap().get(uri).copied();
        let text = String::from_utf8(content.bytes).ok()?;
        match id {
            Some(source_id) => Some(SourceText::new_with_id(text, source_id)),
            None => Some(SourceText::new(text)),
        }
    }

    fn get_uri(&self, id: SourceId) -> Option<Arc<str>> {
        self.ids.read().unwrap().get(&id).cloned()
    }

    fn get_id(&self, uri: &str) -> Option<SourceId> {
        self.uri_to_id.read().unwrap().get(uri).cloned()
    }

    fn exists(&self, uri: &str) -> bool {
        self.overlay.read(Path::new(uri)).is_some()
    }

    fn metadata(&self, uri: &str) -> Option<FileMetadata> {
        let content = self.overlay.read(Path::new(uri))?;
        let modified = self.modified.read().unwrap().get(uri).copied();
        Some(FileMetadata { file_type: FileType::File, len: content.bytes.len() as u64, modified })
    }

    fn read_dir(&self, _uri: &str) -> Option<Vec<Arc<str>>> {
        None
    }
}

impl WritableVfs for MemoryVfs {
    fn write_file(&self, uri: &str, content: Arc<str>) {
        MemoryVfs::write_file(self, uri, content);
    }

    fn remove_file(&self, uri: &str) {
        MemoryVfs::remove_file(self, uri);
    }
}
