//! Public formatter API for AWSL source text.
//!
//! Product entry: [`format_source`] with [`FormatOptions`]. Markup gap rules live in private submodules.

mod bridge;
mod engine;
mod error;
mod options;
mod source;

pub use bridge::AwslCstFormatter;
pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
