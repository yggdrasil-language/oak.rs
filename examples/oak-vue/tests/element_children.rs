use oak_core::{Builder, ParseSession, SourceText};
use oak_vue::{VueBuilder, VueNode};

fn element_children(src: &str) -> Vec<VueNode> {
    let source = SourceText::new(src);
    let mut session = ParseSession::default();
    let built = Builder::build(&VueBuilder::new(), &source, &[], &mut session);
    let root = built.result.expect("root");
    let block = root.blocks.first().expect("template block");
    let element = block
        .children
        .clone()
        .into_iter()
        .find_map(|node| match node {
            VueNode::Element(element) => Some(element),
            _ => None,
        })
        .expect("element");
    element.children
}

#[test]
fn comments_are_element_children() {
    // Vue keeps comments as children, which drives `v-html` / `v-text` child errors.
    let children = element_children("<template><p><!-- note -->x</p></template>");
    assert!(matches!(children.first(), Some(VueNode::Comment(text)) if text == " note "), "expected a comment child");
    assert_eq!(children.len(), 2);
}

#[test]
fn comments_split_text_runs() {
    // The comment markers must not stay in the rendered text.
    let children = element_children("<template><p>a<!-- note -->b</p></template>");
    assert_eq!(children.len(), 3);
    assert!(matches!(children.first(), Some(VueNode::Text(text)) if text.span.start == 13 && text.span.end == 14));
    assert!(matches!(children[1], VueNode::Comment(ref text) if text == " note "));
    assert!(matches!(children[2], VueNode::Text(ref text) if text.span.start == 27 && text.span.end == 28));
}

#[test]
fn whitespace_only_text_is_not_a_child() {
    let children = element_children("<template><p>  </p></template>");
    assert!(children.is_empty());
}
