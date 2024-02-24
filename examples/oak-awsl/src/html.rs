//! HTML void-element helpers for AWSL parsing.

/// HTML elements that do not require a closing tag.
pub const HTML_VOID_ELEMENTS: &[&str] =
    &["area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source", "track", "wbr"];

/// Returns true when `tag` is a void HTML element.
///
/// Only lowercase tags match: AWSL uses PascalCase for components (e.g. `<Link>`),
/// which must never be treated as the lowercase HTML void element `<link>`.
pub fn is_html_void_element(tag: &str) -> bool {
    HTML_VOID_ELEMENTS.iter().any(|name| *name == tag)
}
