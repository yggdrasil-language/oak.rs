//! Virtual File System (VFS) implementations and management.
//!
//! This module provides different implementations of the [`Vfs`](crate::Vfs) trait,
//! including in-memory and disk-based storage.

/// Disk-based VFS implementation using the local file system.
#[cfg(feature = "disk")]
pub mod disk;
/// In-memory VFS implementation.
pub mod memory;
/// File system watcher for tracking changes on disk.
#[cfg(feature = "disk")]
pub mod watch;

#[cfg(feature = "disk")]
pub use disk::DiskVfs;
pub use memory::MemoryVfs;
#[cfg(feature = "disk")]
pub use watch::{DiskWatcher, VfsEvent, VfsRescanReason, VfsWatchError, VfsWatcher};
