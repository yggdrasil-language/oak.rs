use oak_core::{Parser, RedNode, RedTree, SourceText, parser::session::ParseSession};
use oak_html::{HtmlLanguage, HtmlParser, parser::element_type::HtmlElementType};

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
fn void_elements_do_not_swallow_following_siblings() {
    const SOURCE: &str = r#"<p><img src="a.png"><span>b</span></p><div>tail</div>"#;
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
    assert_eq!(element_tags.iter().filter(|tag| tag.starts_with(r#"<img src="a.png">"#)).count(), 1, "{element_tags:?}");
    assert_eq!(element_tags.iter().filter(|tag| tag.starts_with("<span")).count(), 1, "{element_tags:?}");
    assert_eq!(element_tags.iter().filter(|tag| tag.starts_with("<div")).count(), 1, "{element_tags:?}");
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

fn top_level_element_texts(text: &str) -> Vec<String> {
    let language = HtmlLanguage::default();
    let source = SourceText::new(text);
    let mut cache = ParseSession::<HtmlLanguage>::default();
    let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
    let green = parsed.result.expect("parse html");
    let root = RedNode::new(&green, 0);
    root.children()
        .filter_map(|child| match child {
            RedTree::Node(node) if node.element_type() == HtmlElementType::Element => Some(node.text(&source).to_string()),
            _ => None,
        })
        .collect()
}

#[test]
fn self_closing_svg_children_do_not_swallow_following_siblings() {
    const SVG: &str = r#"<svg viewBox="0 0 10 10"><rect width="4" height="4"/><text>label</text></svg>"#;
    let source = format!("<p>before</p>{SVG}<p>after</p>");
    assert_eq!(top_level_element_texts(&source), vec!["<p>before</p>".to_string(), SVG.to_string(), "<p>after</p>".to_string()]);
}

#[test]
fn inline_svg_stays_inside_paragraph() {
    const SOURCE: &str = r#"<p>Status <svg viewBox="0 0 10 10"><circle cx="5" cy="5" r="4"/></svg> online</p>"#;
    assert_eq!(top_level_element_texts(SOURCE), vec![SOURCE.to_string()]);
}

#[test]
fn self_closing_svg_root_does_not_swallow_following_siblings() {
    const SOURCE: &str = r#"<p>a<svg/>b</p><p>c</p>"#;
    assert_eq!(top_level_element_texts(SOURCE), vec![r#"<p>a<svg/>b</p>"#.to_string(), "<p>c</p>".to_string()]);
}

#[test]
fn self_closing_math_children_do_not_swallow_following_siblings() {
    const SOURCE: &str = r#"<p>a<math><mi>x</mi><mspace width="1em"/></math>b</p><p>c</p>"#;
    assert_eq!(top_level_element_texts(SOURCE), vec![r#"<p>a<math><mi>x</mi><mspace width="1em"/></math>b</p>"#.to_string(), "<p>c</p>".to_string()]);
}
