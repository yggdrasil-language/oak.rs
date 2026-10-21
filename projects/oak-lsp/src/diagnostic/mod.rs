//! UTF-16 position mapping and structured diagnostic conversion for LSP clients.

mod convert;
mod error;
mod position;

pub use boost_diagnostic::{
    self,
    terminal::{SourceCache, SourceID, SourcePath, SourceProvider, SourceSpan},
};
pub use convert::{SourceResolver, structured_set_to_lsp, structured_to_lsp};
pub use error::DiagnosticError;
pub use lsp_types;
pub use position::{byte_index_to_position, byte_range_to_lsp_range, byte_span_to_range, position_to_byte_index, range_to_byte_span};
