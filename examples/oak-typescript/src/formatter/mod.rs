//! Public formatter API for TypeScript/JavaScript source text.
//!
//! Product entry: [`format_source`] with [`FormatOptions`]. CST rules and
//! transitional AST output stay in private submodules.

mod bridge;
pub(crate) mod cst_options;
mod cst_source;
mod engine;
mod error;
mod options;
mod red_tree;
pub(crate) mod trivia_guard;

pub use engine::format_source;
pub use error::FormatError;
pub use options::FormatOptions;

pub(crate) use cst_options::CstFormatOptions;
