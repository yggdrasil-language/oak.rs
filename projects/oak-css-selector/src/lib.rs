#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

//! CSS selector subset DSL for Oak document queries.
//!
//! Parsing and AST ownership live here. Execution against HTML trees is provided
//! by `oak-html` using DOM-compatible `ElementView` adapters.

mod ast;
mod diag;
mod parser;

pub use ast::{
    AttributeOperator, AttributeSelector, Combinator, CompoundSelector, Selector, SelectorList,
    SimpleSelector,
};
pub use diag::CssSelectorParseError;
pub use parser::parse_css_selector;
