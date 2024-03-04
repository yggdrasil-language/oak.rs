#![doc = include_str!("readme.md")]
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
