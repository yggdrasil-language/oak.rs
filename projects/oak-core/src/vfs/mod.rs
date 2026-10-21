//! Virtual file system and document session for Oak language services.

use crate::{
    Arc,
    source::{LineMap, Source, SourceId},
};

mod document;
mod memory;
mod revision;

#[cfg(feature = "vfs-disk")]
mod disk;
#[cfg(feature = "vfs-disk")]
mod watch;

pub use document::DocumentSession;
pub use memory::MemoryVfs;
pub use revision::RevisionTracker;

#[cfg(feature = "vfs-disk")]
pub use disk::DiskVfs;
#[cfg(feature = "vfs-disk")]
pub use watch::{DiskWatcher, VfsEvent, VfsRescanReason, VfsWatchError, VfsWatcher};

/// Type of a file in the VFS.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FileType {
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// Other file types (e.g., symbolic links, sockets).
    Other,
}

/// Metadata for a file or directory in the VFS.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct FileMetadata {
    /// The type of the file.
    pub file_type: FileType,
    /// The length of the file in bytes.
    pub len: u64,
    /// The last modification time as a Unix timestamp in seconds.
    pub modified: Option<u64>,
}

/// A trait for a Virtual File System that can provide source content and location mapping.
pub trait Vfs: Send + Sync {
    /// The type of source returned by this VFS.
    type Source: Source + 'static;

    /// Get the source for the given URI.
    fn get_source(&self, uri: &str) -> Option<Self::Source>;

    /// Get the URI for the given SourceId.
    fn get_uri(&self, id: SourceId) -> Option<Arc<str>>;

    /// Get the SourceId for the given URI.
    fn get_id(&self, uri: &str) -> Option<SourceId>;

    /// Check if a path exists at the given URI.
    fn exists(&self, uri: &str) -> bool;

    /// Read the metadata for the given URI.
    fn metadata(&self, uri: &str) -> Option<FileMetadata>;

    /// Read the contents of a directory at the given URI.
    fn read_dir(&self, uri: &str) -> Option<Vec<Arc<str>>>;

    /// Check if the given URI points to a file.
    fn is_file(&self, uri: &str) -> bool {
        self.metadata(uri).map(|m| m.file_type == FileType::File).unwrap_or(false)
    }

    /// Check if the given URI points to a directory.
    fn is_dir(&self, uri: &str) -> bool {
        self.metadata(uri).map(|m| m.file_type == FileType::Directory).unwrap_or(false)
    }

    /// Get a [`LineMap`] for the given URI.
    fn line_map(&self, uri: &str) -> Option<LineMap> {
        self.get_source(uri).map(|s| LineMap::from_source(&s))
    }
}

/// A trait for a Virtual File System that supports writing.
pub trait WritableVfs: Vfs {
    /// Update or create a file with the given content.
    fn write_file(&self, uri: &str, content: Arc<str>);

    /// Remove a file from the VFS.
    fn remove_file(&self, uri: &str);
}
