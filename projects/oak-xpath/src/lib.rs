#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

//! XPath subset DSL for Oak document queries.
//!
//! Parsing and AST ownership live here. Execution against XML trees is provided
//! by `oak-xml` using namespace-aware `ElementView` adapters.

mod ast;
mod diag;
mod parser;

pub use ast::{Axis, NodeTest, PathExpr, Predicate, QName, Step, XpathExpr};
pub use diag::XpathParseError;
pub use parser::parse_xpath;
