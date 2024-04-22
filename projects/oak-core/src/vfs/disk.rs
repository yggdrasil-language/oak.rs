use boost_fs::vfs::LayeredFs;
use crate::{
    Arc,
    source::{DocumentSnapshot, SourceId, SourceText},
};
use super::{DocumentSession, FileMetadata, FileType, RevisionTracker, Vfs, WritableVfs};
use std::{collections::HashMap, path::PathBuf, sync::RwLock};

/// A virtual file system that reads from disk through [`boost_fs::vfs::LayeredFs`].
pub struct DiskVfs {
    fs: LayeredFs,
    revisions: RevisionTracker,
    /// Mapping from file URIs to internal source IDs.
    uri_to_id: RwLock<HashMap<Arc<str>, SourceId>>,
    /// Mapping from source IDs back to file URIs.
    id_to_uri: RwLock<HashMap<SourceId, Arc<str>>>,
    /// Base path for relative URIs.
    root_path: PathBuf,
}

impl DiskVfs {
    /// Creates a new DiskVfs instance with the given root path.
    pub fn new(root_path: PathBuf) -> Self {
        Self {
            fs: LayeredFs::new(root_path.clone()),
            revisions: RevisionTracker::new(),
            uri_to_id: RwLock::new(HashMap::new()),
            id_to_uri: RwLock::new(HashMap::new()),
            root_path,
        }
    }

    /// Returns the underlying layered filesystem for overlay updates.
    pub fn layered(&self) -> &LayeredFs {
        &self.fs
    }

    /// Returns the revision tracker for open documents.
    pub fn revisions(&self) -> &RevisionTracker {
        &self.revisions
    }

    fn snapshot_after_write(&self, uri: &str) -> DocumentSnapshot {
        let revision = self.revisions.get(uri).expect("revision was just assigned");
        let source = self.get_source(uri).expect("document was just written");
        DocumentSnapshot::new(source, revision)
    }

    /// Converts a URI to a physical file path.
    fn uri_to_path(&self, uri: &str) -> Option<PathBuf> {
        let path = PathBuf::from(uri);
        if path.is_absolute() { Some(path) } else { Some(self.root_path.join(path)) }
    }

    fn vfs_path(&self, uri: &str) -> Option<PathBuf> {
        let path = PathBuf::from(uri);
        if path.is_absolute() { path.strip_prefix(&self.root_path).ok().map(|p| p.to_path_buf()) } else { Some(path) }
    }

    /// Converts a physical file path to a URI.
    fn path_to_uri(&self, path: PathBuf) -> Arc<str> {
        let uri = if path.is_absolute() {
            path.to_string_lossy().to_string()
        }
        else {
            path.strip_prefix(&self.root_path).unwrap_or(&path).to_string_lossy().to_string()
        };
        Arc::from(uri.as_str())
    }
}

impl Vfs for DiskVfs {
    type Source = SourceText;

    /// Reads the content of a file from the layered filesystem.
    fn get_source(&self, uri: &str) -> Option<Self::Source> {
        let vfs_path = self.vfs_path(uri)?;
        let content = self.fs.read(vfs_path.as_path()).ok()?;
        let text = String::from_utf8(content.bytes).ok()?;
        Some(SourceText::new(text))
    }

    /// Returns the URI for a given source ID.
    fn get_uri(&self, id: SourceId) -> Option<Arc<str>> {
        let id_to_uri = self.id_to_uri.read().unwrap();
        id_to_uri.get(&id).cloned()
    }

    /// Returns the source ID for a given URI, if it exists.
    fn get_id(&self, uri: &str) -> Option<SourceId> {
        let uri_to_id = self.uri_to_id.read().unwrap();
        uri_to_id.get(uri).cloned()
    }

    /// Checks if a file or directory exists at the given URI.
    fn exists(&self, uri: &str) -> bool {
        self.vfs_path(uri).map(|path| self.fs.exists(path.as_path())).unwrap_or(false)
    }

    /// Retrieves metadata for a file or directory from the disk layer.
    fn metadata(&self, uri: &str) -> Option<FileMetadata> {
        let path = self.uri_to_path(uri)?;
        let meta = boost_fs::metadata(&path).ok()?;

        let file_type = if meta.is_file() {
            FileType::File
        }
        else if meta.is_dir() {
            FileType::Directory
        }
        else {
            FileType::Other
        };

        let modified = meta.modified().ok().and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok()).map(|d| d.as_secs());

        Some(FileMetadata { file_type, len: meta.len(), modified })
    }

    /// Reads the contents of a directory on disk and returns their URIs.
    fn read_dir(&self, uri: &str) -> Option<Vec<Arc<str>>> {
        let vfs_path = self.vfs_path(uri)?;
        let entries = self.fs.read_dir(vfs_path.as_path()).ok()?;
        Some(entries.into_iter().map(|path| self.path_to_uri(path)).collect())
    }
}

impl DocumentSession for DiskVfs {
    fn open_document(&self, uri: &str, text: Arc<str>) -> DocumentSnapshot {
        self.write_file(uri, text);
        self.snapshot_after_write(uri)
    }

    fn close_document(&self, uri: &str) {
        self.revisions.clear(uri);
        if let Some(vfs_path) = self.vfs_path(uri) {
            self.fs.clear_overlay(vfs_path.as_path());
        }
    }

    fn document_snapshot(&self, uri: &str) -> Option<DocumentSnapshot> {
        let revision = self.revisions.get(uri)?;
        self.get_source(uri).map(|source| DocumentSnapshot::new(source, revision))
    }

    fn request_source(&self, uri: &str) -> Option<SourceText> {
        self.document_snapshot(uri).map(|snapshot| snapshot.source().clone()).or_else(|| self.get_source(uri))
    }
}

impl WritableVfs for DiskVfs {
    /// Writes unsaved content into the overlay and persists to disk.
    fn write_file(&self, uri: &str, content: Arc<str>) {
        if let Some(path) = self.uri_to_path(uri) {
            if let Some(parent) = path.parent() {
                let _ = boost_fs::create_folder_all(parent);
            }
            let _ = boost_fs::write(&path, content.as_bytes());
            if let Some(vfs_path) = self.vfs_path(uri) {
                self.fs.write_overlay(vfs_path, content.as_bytes().to_vec());
            }
        }
        self.revisions.bump(uri);
    }

    /// Removes a file from disk and clears overlay state.
    fn remove_file(&self, uri: &str) {
        if let Some(path) = self.uri_to_path(uri) {
            let _ = boost_fs::remove_file(&path);
            if let Some(vfs_path) = self.vfs_path(uri) {
                self.fs.remove_overlay(vfs_path.as_path());
            }
        }
        self.revisions.clear(uri);
    }
}
