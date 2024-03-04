//! HTML document queries via CSS selector DSL.
//!
//! Selector parsing lives in `oak-css-selector`. Execution uses HTML-compatible element views.

mod css;
mod view;

pub use css::{select_css, select_css_elements};
pub use view::{HtmlDocumentView, HtmlElementView};
