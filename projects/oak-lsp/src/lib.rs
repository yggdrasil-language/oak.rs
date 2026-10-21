#![feature(new_range_api)]
#![recursion_limit = "512"]
#![warn(missing_docs)]
#![doc = "Language Server Protocol (LSP) implementation for Oak languages."]

/// Structured diagnostic conversion for LSP clients.
pub mod diagnostic;
/// Handlers for LSP requests and notifications.
pub mod handlers;
/// `Lsp` contract that dispatches protocol messages to Oak language capabilities.
pub mod lsp;
/// LSP protocol messages, methods and capabilities.
pub mod protocol;
/// Language service trait and utilities.
pub mod service;
/// Transport adapters and the driver loop, orthogonal to server providers.
pub mod transport;
/// LSP-specific type definitions.
pub mod types;
/// Workspace and file management for LSP.
pub mod workspace;

pub use handlers::*;
pub use lsp::Lsp;
pub use oak_vfs::{FileMetadata, FileType, MemoryVfs, Vfs};
pub use protocol::*;
pub use service::LanguageService;
pub use transport::{Driver, DriverError, Inbound, MessageTransport, Outbound, RequestId};
pub use types::*;
pub use workspace::WorkspaceManager;
