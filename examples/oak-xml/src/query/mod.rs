//! XML document queries via XPath subset DSL.
//!
//! Selector parsing lives in `oak-xpath`. Execution uses namespace-aware element views.

mod view;
mod xpath;

#[cfg(test)]
mod xpath_tests;

pub use view::{XmlDocumentView, XmlElementView};
pub use xpath::{XmlNamespaceContext, select_xpath, select_xpath_elements};
