#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

use oak_core::{
    Arc,
    source::{Source, SourceId},
};

mod line_map;
pub use line_map::LineMap;

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

/// Virtual File System implementation and utilities.
pub mod vfs;
pub use vfs::MemoryVfs;
#[cfg(feature = "disk")]
pub use vfs::{DiskVfs, DiskWatcher, VfsEvent, VfsRescanReason, VfsWatchError, VfsWatcher};

/// A trait for a Virtual File System that can provide source content and location mapping.
///
/// The `Vfs` trait provides a unified abstraction for file system operations,
/// allowing the core framework to work with files from disk, memory, or network
/// without being tied to a specific storage backend.
///
/// # Usage Scenario
///
/// The `Vfs` is typically used to:
/// 1. Resolve URIs to source content via [`get_source`].
/// 2. Map between [`SourceId`] and URI strings.
/// 3. Provide metadata and directory listing for code navigation and discovery.
/// 4. Manage [`LineMap`]s for translating byte offsets to line/column positions.
pub trait Vfs: Send + Sync {
    /// The type of source returned by this VFS.
    type Source: Source + 'static;

    /// Get the source for the given URI.
    ///
    /// # Arguments
    /// * `uri` - The URI of the file to retrieve.
    fn get_source(&self, uri: &str) -> Option<Self::Source>;

    /// Get the URI for the given SourceId.
    ///
    /// # Arguments
    /// * `id` - The ID of the source to retrieve the URI for.
    fn get_uri(&self, id: SourceId) -> Option<Arc<str>>;

    /// Get the SourceId for the given URI.
    ///
    /// # Arguments
    /// * `uri` - The URI to retrieve the ID for.
    fn get_id(&self, uri: &str) -> Option<SourceId>;

    /// Check if a path exists at the given URI.
    ///
    /// # Arguments
    /// * `uri` - The URI to check for existence.
    fn exists(&self, uri: &str) -> bool;

    /// Read the metadata for the given URI.
    ///
    /// # Arguments
    /// * `uri` - The URI to retrieve metadata for.
    fn metadata(&self, uri: &str) -> Option<FileMetadata>;

    /// Read the contents of a directory at the given URI.
    ///
    /// # Arguments
    /// * `uri` - The URI of the directory to read.
    ///
    /// # Returns
    /// A list of URIs or names within the directory.
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
    ///
    /// # Arguments
    /// * `uri` - The URI to get the line map for.
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
