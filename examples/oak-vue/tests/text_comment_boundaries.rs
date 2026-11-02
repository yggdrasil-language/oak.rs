//! `//` / `/* … */` are JavaScript comments; they only exist where JavaScript
//! lives (`<script>` / `<style>` content, interpolations, directive
//! expressions). Plain template text treats them as literal characters, so
//! `serve public/** assets` or a `https://` URL can never swallow the rest of
//! the template as a "comment".

use oak_core::{Builder, ParseSession, Source, SourceText};
use oak_vue::{VueBuilder, VueNode};

struct Built {
    root: oak_vue::VueRoot,
    source: String,
}

fn build(src: &str) -> Built {
    let source = SourceText::new(src);
    let mut session = ParseSession::default();
    let built = Builder::build(&VueBuilder::new(), &source, &[], &mut session);
    assert!(built.diagnostics.is_empty(), "{src}: {:#?}", built.diagnostics);
    Built { root: built.result.expect("root"), source: src.to_string() }
}

fn text_of(built: &Built, span: core::range::Range<usize>) -> String {
    SourceText::new(built.source.clone()).get_text_in(span).to_string()
}

/// Collects the text content of every `VueNode::Text` in the first element.
fn first_element_texts(built: &Built) -> Vec<String> {
    let block = built.root.blocks.first().expect("template block");
    let element = block
        .children
        .iter()
        .find_map(|node| match node {
            VueNode::Element(e) => Some(e),
            _ => None,
        })
        .expect("element");
    element
        .children
        .iter()
        .filter_map(|n| match n {
            VueNode::Text(t) => Some(text_of(built, t.span.clone())),
            _ => None,
        })
        .collect()
}

fn first_element(built: &Built) -> &oak_vue::VueElement {
    let block = built.root.blocks.first().expect("template block");
    block
        .children
        .iter()
        .find_map(|node| match node {
            VueNode::Element(e) => Some(e),
            _ => None,
        })
        .expect("element")
}

#[test]
fn slash_star_star_in_text_stays_literal() {
    let built = build("<template><p>serve public/** assets</p></template>");
    assert_eq!(first_element_texts(&built), vec!["serve public/** assets"]);
}

#[test]
fn closed_block_comment_marks_stay_literal_text() {
    let built = build("<template><p>a /* b */ c</p></template>");
    let texts = first_element_texts(&built);
    assert_eq!(texts.join(""), "a /* b */ c");
    assert!(!texts.iter().any(|t| t.contains('<')), "text must not absorb the closing tag: {texts:?}");
}

#[test]
fn line_comment_marks_in_text_stay_literal() {
    let built = build("<template><p>a // b</p></template>");
    assert_eq!(first_element_texts(&built).join(""), "a // b");
}

#[test]
fn lone_comment_close_in_text_stays_literal() {
    let built = build("<template><p>a */ b</p></template>");
    assert_eq!(first_element_texts(&built).join(""), "a */ b");
}

#[test]
fn url_scheme_slashes_in_text_stay_literal() {
    let built = build("<template><p>see https://example.com/docs</p></template>");
    assert_eq!(first_element_texts(&built).join(""), "see https://example.com/docs");
}

#[test]
fn glob_star_slash_mix_in_text_stays_literal() {
    let built = build("<template><p>path/**/file and */ trailing</p></template>");
    assert_eq!(first_element_texts(&built).join(""), "path/**/file and */ trailing");
}

#[test]
fn html_comment_still_becomes_comment_node() {
    let built = build("<template><p>a<!-- note -->b</p></template>");
    let element = first_element(&built);
    assert!(element.children.iter().any(|n| matches!(n, VueNode::Comment(text) if text == " note ")), "html comment must stay a comment node: {:?}", element.children);
}

#[test]
fn interpolation_still_allows_js_comments() {
    let built = build("<template><p>{{ a /* note */ + b }}</p></template>");
    let element = first_element(&built);
    assert!(element.children.iter().any(|n| matches!(n, VueNode::Interpolation(_))), "interpolation must survive: {:?}", element.children);
}

#[test]
fn script_content_keeps_js_comments() {
    let built = build("<template><p>x</p></template><script>const a = 1; // note\nconst b = 2; /* tail */</script>");
    assert!(built.root.blocks.iter().any(|b| text_of(&built, b.name.clone()) == "script"));
}

#[test]
fn text_after_comment_marks_still_interpolates() {
    let built = build("<template><p>public/** then {{ value }}</p></template>");
    let element = first_element(&built);
    let has_text = element.children.iter().any(|n| matches!(n, VueNode::Text(t) if text_of(&built, t.span.clone()).contains("public/**")));
    let has_interpolation = element.children.iter().any(|n| matches!(n, VueNode::Interpolation(_)));
    assert!(has_text && has_interpolation, "{:?}", element.children);
}
