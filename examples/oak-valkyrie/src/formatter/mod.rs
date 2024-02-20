//! Public formatter API for Valkyrie source text.
//!
//! Product entry: [`format_source`] with [`FormatOptions`]. CST rules live in private submodules.

mod bridge;
mod engine;
mod error;
mod options;
mod source;

pub use bridge::ValkyrieCstFormatter;
pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
