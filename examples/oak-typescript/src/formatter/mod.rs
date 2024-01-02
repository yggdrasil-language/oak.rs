//! Public formatter API for TypeScript/JavaScript source text.
//!
//! Product entry: [`format_source`] with [`FormatOptions`]. CST rules live in private submodules.

mod bridge;
mod engine;
mod error;
mod options;
mod source;
pub(crate) mod trivia_guard;

pub use bridge::TypeScriptCstFormatter;
pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;
