//! Public formatter API for Vue SFC source text.
//!
//! Product entry: [`format_source`] with [`FormatOptions`].

mod engine;
mod error;
mod options;

pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
