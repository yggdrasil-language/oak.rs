use oak_core::{Parser, RedNode, RedTree, SourceText, parser::session::ParseSession};
use oak_html::{HtmlLanguage, HtmlParser, parser::element_type::HtmlElementType};

use oak_core::{Parser, RedNode, RedTree, SourceText, parser::session::ParseSession};

#[test]
fn parse_simple_anchor() {
    const SOURCE: &str = r#"<a href="part.xhtml">Part One</a>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
    let green = parsed.result.expect("parse html");
    let root = RedNode::new(&green, 0);
    let mut element_tags = Vec::new();
    fn walk(node: RedNode<'_, HtmlLanguage>, source: &SourceText, out: &mut Vec<String>) {
        if node.element_type() == HtmlElementType::Element {
            out.push(node.text(source).to_string());
        }
        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                walk(child_node, source, out);
            }
        }
    }
    for child in root.children() {
        if let RedTree::Node(child_node) = child {
            walk(child_node, &source, &mut element_tags);
        }
    }
    assert_eq!(element_tags, vec![SOURCE.to_string()]);
}

#[test]
fn parse_li_with_nested_anchor_and_list() {
    const SOURCE: &str = r#"<li><a href="part.xhtml">Part One</a><ol><li><a href="part.xhtml#ch1">Chapter 1</a></li></ol></li>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
    let green = parsed.result.expect("parse html");
    let root = RedNode::new(&green, 0);
    let mut element_tags = Vec::new();
    fn walk(node: RedNode<'_, HtmlLanguage>, source: &SourceText, out: &mut Vec<String>) {
        if node.element_type() == HtmlElementType::Element {
            out.push(node.text(source).to_string());
        }
        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                walk(child_node, source, out);
            }
        }
    }
    for child in root.children() {
        if let RedTree::Node(child_node) = child {
            walk(child_node, &source, &mut element_tags);
        }
    }
    assert_eq!(element_tags.iter().filter(|tag| tag.starts_with("<a")).count(), 2, "anchors: {element_tags:?}");
}

#[test]
fn parse_nested_nav_green_structure() {
    const SOURCE: &str = r#"<nav epub:type="toc">
<ol>
  <li><a href="part.xhtml">Part One</a>
<ol>
  <li><a href="part.xhtml#ch1">Chapter 1</a></li>
</ol>
  </li>
</ol>
</nav>"#;
    let language = HtmlLanguage::default();
    let source = SourceText::new(SOURCE);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
    let green = parsed.result.expect("parse html");
    let root = RedNode::new(&green, 0);
    let mut anchor_count = 0usize;
    fn walk(node: RedNode<'_, HtmlLanguage>, source: &SourceText, count: &mut usize) {
        if node.element_type() == HtmlElementType::Element {
            let text = node.text(source);
            if text.starts_with("<a") {
                *count += 1;
            }
        }
        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                walk(child_node, source, count);
            }
        }
    }
    for child in root.children() {
        if let RedTree::Node(child_node) = child {
            walk(child_node, &source, &mut anchor_count);
        }
    }
    assert_eq!(anchor_count, 2);
}
