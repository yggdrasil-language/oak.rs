//! Public formatter API for TypeScript/JavaScript source text.
//!
//! Product entry: [`format_source`] with [`FormatOptions`]. Implementation modules
//! (`cst_format`, `print`, `red_tree`) stay private.

mod engine;
mod error;
mod options;

pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
