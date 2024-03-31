#![feature(new_range_api)]
#![recursion_limit = "512"]
#![warn(missing_docs)]
#![doc = "Language Server Protocol (LSP) implementation for Oak languages."]

/// Handlers for LSP requests and notifications.
pub mod handlers;
/// LSP protocol messages, methods and capabilities.
pub mod protocol;
/// Language service trait and utilities.
pub mod service;
/// LSP-specific type definitions.
pub mod types;
/// Workspace and file management for LSP.
pub mod workspace;

pub use handlers::*;
pub use oak_vfs::{FileMetadata, FileType, MemoryVfs, Vfs};
pub use protocol::*;
pub use service::LanguageService;
pub use types::*;
pub use workspace::WorkspaceManager;
