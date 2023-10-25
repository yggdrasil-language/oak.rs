//! Public `format` API for TypeScript/JavaScript source text.
//!
//! This is the only supported product entry for formatting in `oak-typescript`.
//! Tree-aware and AST-based implementation details remain private crate modules.

mod engine;
mod error;
mod options;

pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
