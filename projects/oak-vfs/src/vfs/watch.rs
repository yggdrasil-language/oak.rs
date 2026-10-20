//! Disk change observation for [`DiskVfs`](super::DiskVfs).
//!
//! [`DiskWatcher`] owns an [`fs_tools`] watcher and converts reported paths into the same URIs
//! that [`DiskVfs`](super::DiskVfs) uses. Subscriptions are recursive.

use std::{
    error::Error,
    fmt,
    path::{Path, PathBuf},
};

use fs_tools::watch::{RescanReason, WatchError, WatchEvent, WatchOptions, Watcher};

/// An event emitted by a [`DiskWatcher`].
///
/// Paths are reported as URIs relative to the watcher root when they are below it, and as absolute
/// URIs otherwise. Consumers own version advancement and source invalidation.
#[derive(Debug)]
pub enum VfsEvent {
    /// A file or directory was modified.
    Changed(String),
    /// A file or directory was created.
    Created(String),
    /// A file or directory was removed.
    Removed(String),
    /// The subtree at `root` may not match the events already delivered.
    ///
    /// Re-read that subtree before trusting cached sources.
    Rescan {
        /// URI of the subtree whose state is uncertain.
        root: String,
        /// Why the subtree must be re-read.
        reason: VfsRescanReason,
    },
    /// The watcher reported a failure.
    ///
    /// Every error except [`VfsWatchError::Watch`] wrapping a fatal backend failure leaves the
    /// watcher running. A [`VfsWatchError::NonUnicodePath`] means the event stream cannot describe
    /// the path, so the whole watched root must be re-read.
    Error(VfsWatchError),
}

/// Why a [`VfsEvent::Rescan`] was emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VfsRescanReason {
    /// The event queue overflowed and events were dropped.
    Overflow,
    /// A directory was registered after its contents appeared.
    NewDirectory,
    /// A rename was observed without a reliable pair.
    UnpairedRename,
}

impl From<RescanReason> for VfsRescanReason {
    fn from(reason: RescanReason) -> Self {
        match reason {
            RescanReason::Overflow => Self::Overflow,
            RescanReason::NewDirectory => Self::NewDirectory,
            RescanReason::UnpairedRename => Self::UnpairedRename,
        }
    }
}

/// A failure while creating, controlling, or running a [`DiskWatcher`].
#[derive(Debug)]
pub enum VfsWatchError {
    /// The underlying watcher rejected or reported the failure.
    Watch(WatchError),
    /// An event path cannot be represented as a UTF-8 URI.
    ///
    /// Re-read the whole watched root when this is received.
    NonUnicodePath(PathBuf),
}

impl From<WatchError> for VfsWatchError {
    fn from(error: WatchError) -> Self {
        Self::Watch(error)
    }
}

impl fmt::Display for VfsWatchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Watch(error) => error.fmt(f),
            Self::NonUnicodePath(path) => write!(f, "watched path is not valid UTF-8: {}", path.display()),
        }
    }
}

impl Error for VfsWatchError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Watch(error) => Some(error),
            Self::NonUnicodePath(_) => None,
        }
    }
}

/// A trait for watching file system changes.
pub trait VfsWatcher {
    /// Starts watching the given URI recursively.
    fn watch(&mut self, uri: &str) -> Result<(), VfsWatchError>;
    /// Stops watching the given URI and everything below it.
    fn unwatch(&mut self, uri: &str) -> Result<(), VfsWatchError>;
    /// Stops watching every URI.
    fn unwatch_all(&mut self) -> Result<(), VfsWatchError>;
}

/// A watcher implementation that watches files on disk.
///
/// `root_path` is the same base path used by [`DiskVfs`](super::DiskVfs). Relative URIs are joined
/// to it, and absolute paths below it are reported as relative URIs.
pub struct DiskWatcher {
    watcher: Watcher,
    root_path: PathBuf,
}

impl DiskWatcher {
    /// Creates a new `DiskWatcher` with the given root path and callback.
    pub fn new<F>(root_path: impl Into<PathBuf>, callback: F) -> Result<Self, VfsWatchError>
    where
        F: FnMut(VfsEvent) + Send + 'static,
    {
        Self::with_options(root_path, WatchOptions::new(), callback)
    }

    /// Creates a new `DiskWatcher` with explicit queue and backend options.
    pub fn with_options<F>(root_path: impl Into<PathBuf>, options: WatchOptions, callback: F) -> Result<Self, VfsWatchError>
    where
        F: FnMut(VfsEvent) + Send + 'static,
    {
        let root_path = root_path.into();
        let mapper = UriMapper { root_path: root_path.clone() };
        let mut callback = callback;
        let watcher = Watcher::new(move |event| translate(&mapper, event, &mut callback), options)?;
        Ok(Self { watcher, root_path })
    }

    /// Converts a URI to a physical path, mirroring [`DiskVfs`](super::DiskVfs).
    fn uri_to_path(&self, uri: &str) -> PathBuf {
        let path = PathBuf::from(uri);
        if path.is_absolute() { path } else { self.root_path.join(path) }
    }

    /// Stops the watcher and reports failures.
    ///
    /// Dropping the watcher performs the same release without reporting errors.
    pub fn close(self) -> Result<(), VfsWatchError> {
        self.watcher.close().map_err(Into::into)
    }
}

impl VfsWatcher for DiskWatcher {
    fn watch(&mut self, uri: &str) -> Result<(), VfsWatchError> {
        self.watcher.watch(self.uri_to_path(uri)).map_err(Into::into)
    }

    fn unwatch(&mut self, uri: &str) -> Result<(), VfsWatchError> {
        self.watcher.unwatch(self.uri_to_path(uri)).map_err(Into::into)
    }

    fn unwatch_all(&mut self) -> Result<(), VfsWatchError> {
        self.watcher.unwatch_all().map_err(Into::into)
    }
}

/// Maps reported paths to VFS URIs.
struct UriMapper {
    root_path: PathBuf,
}

impl UriMapper {
    /// Converts a path to a URI using the [`DiskVfs`](super::DiskVfs) rule.
    fn uri(&self, path: &Path) -> Result<String, VfsWatchError> {
        let mapped = path.strip_prefix(&self.root_path).unwrap_or(path);
        mapped.to_str().map(ToOwned::to_owned).ok_or_else(|| VfsWatchError::NonUnicodePath(path.to_path_buf()))
    }
}

/// Delivers one watcher event as one or more VFS events.
fn translate<F>(mapper: &UriMapper, event: WatchEvent, callback: &mut F)
where
    F: FnMut(VfsEvent),
{
    match event {
        WatchEvent::Created(path) => callback(uri_event(mapper, &path, VfsEvent::Created)),
        WatchEvent::Modified(path) => callback(uri_event(mapper, &path, VfsEvent::Changed)),
        WatchEvent::Removed(path) => callback(uri_event(mapper, &path, VfsEvent::Removed)),
        WatchEvent::Renamed { from, to } => {
            callback(uri_event(mapper, &from, VfsEvent::Removed));
            callback(uri_event(mapper, &to, VfsEvent::Created));
        }
        WatchEvent::Rescan { root, reason } => callback(match mapper.uri(&root) {
            Ok(root) => VfsEvent::Rescan { root, reason: reason.into() },
            Err(error) => VfsEvent::Error(error),
        }),
        WatchEvent::Error(error) => callback(VfsEvent::Error(error.into())),
    }
}

/// Builds a URI event, falling back to an error when the path has no UTF-8 representation.
fn uri_event(mapper: &UriMapper, path: &Path, event: impl FnOnce(String) -> VfsEvent) -> VfsEvent {
    match mapper.uri(path) {
        Ok(uri) => event(uri),
        Err(error) => VfsEvent::Error(error),
    }
}
