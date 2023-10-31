use crate::{
    ast::{Attribute, Element, HtmlDocument, HtmlNode, Text},
    lexer::token_type::HtmlTokenType,
    parser::element_type::HtmlElementType,
};
use oak_core::{RedNode, RedTree, SourceText, TokenType};

use super::HtmlLanguage;

/// Lowers a parsed green tree root into a typed HTML document AST.
pub(crate) fn build_document<'a>(green_tree: &'a oak_core::GreenNode<'a, HtmlLanguage>, source: &SourceText) -> HtmlDocument {
    let root = RedNode::new(green_tree, 0);
    let mut nodes = Vec::new();
    for child in root.children() {
        if let Some(node) = build_top_level_node(child, source) {
            nodes.push(node);
        }
    }
    HtmlDocument { nodes }
}

fn build_top_level_node(tree: RedTree<HtmlLanguage>, source: &SourceText) -> Option<HtmlNode> {
    match tree {
        RedTree::Node(node) if node.element_type() == HtmlElementType::Element => build_element_node(node, source),
        _ => None,
    }
}

fn build_element_node(node: RedNode<HtmlLanguage>, source: &SourceText) -> Option<HtmlNode> {
    let tag_name = element_tag_name(node, source)?;
    let attributes = collect_attributes(node, source);
    let mut children = Vec::new();
    for child in element_body_children(node, source) {
        if let Some(child_node) = build_element_child(child, source) {
            children.push(child_node);
        }
    }
    Some(HtmlNode::Element(Element { tag_name, attributes, children, span: node.span() }))
}

fn build_element_child(tree: RedTree<HtmlLanguage>, source: &SourceText) -> Option<HtmlNode> {
    match tree {
        RedTree::Node(node) if node.element_type() == HtmlElementType::Element => build_element_node(node, source),
        RedTree::Node(node) if node.element_type() == HtmlElementType::Text => {
            let content = normalize_whitespace(node.text(source).as_ref());
            if content.is_empty() { None } else { Some(HtmlNode::Text(Text { content, span: node.span() })) }
        }
        RedTree::Leaf(leaf) => match leaf.kind {
            HtmlTokenType::Text | HtmlTokenType::Whitespace | HtmlTokenType::Newline => {
                let content = normalize_whitespace(leaf.text(source).as_ref());
                if content.is_empty() { None } else { Some(HtmlNode::Text(Text { content, span: leaf.span() })) }
            }
            _ => None,
        },
        _ => None,
    }
}

fn collect_attributes(node: RedNode<HtmlLanguage>, source: &SourceText) -> Vec<Attribute> {
    let mut attributes = Vec::new();
    let mut saw_element_name = false;
    let mut pending_name: Option<String> = None;

    for child in node.children() {
        match child {
            RedTree::Node(attr_node) if attr_node.element_type() == HtmlElementType::Attribute => {
                let mut name = None;
                let mut value = None;
                let span = attr_node.span();
                for attr_child in attr_node.children() {
                    match attr_child {
                        RedTree::Node(part) => match part.element_type() {
                            HtmlElementType::AttributeName => {
                                name = Some(part.text(source).trim().to_string());
                            }
                            HtmlElementType::AttributeValue => {
                                value = Some(unquote(part.text(source).as_ref()));
                            }
                            _ => {}
                        },
                        RedTree::Leaf(leaf) => match leaf.kind {
                            HtmlTokenType::AttributeName | HtmlTokenType::TagName => {
                                name = Some(leaf.text(source).trim().to_string());
                            }
                            HtmlTokenType::AttributeValue => {
                                value = Some(unquote(leaf.text(source).as_ref()));
                            }
                            _ => {}
                        },
                    }
                }
                if let Some(name) = name {
                    attributes.push(Attribute { name, value, span });
                }
            }
            RedTree::Leaf(leaf) => match leaf.kind {
                HtmlTokenType::TagName => {
                    if !saw_element_name {
                        saw_element_name = true;
                    }
                    else {
                        pending_name = Some(leaf.text(source).trim().to_string());
                    }
                }
                HtmlTokenType::AttributeName => {
                    pending_name = Some(leaf.text(source).trim().to_string());
                }
                HtmlTokenType::AttributeValue => {
                    if let Some(name) = pending_name.take() {
                        attributes.push(Attribute { name, value: Some(unquote(leaf.text(source).as_ref())), span: leaf.span() });
                    }
                }
                HtmlTokenType::TagClose | HtmlTokenType::TagSelfClose => break,
                _ => {}
            },
            _ => {}
        }
    }

    attributes
}

fn element_body_children<'a>(node: RedNode<'a, HtmlLanguage>, source: &SourceText) -> Vec<RedTree<'a, HtmlLanguage>> {
    let tag_name = element_tag_name(node, source);
    let node_children: Vec<RedTree<'a, HtmlLanguage>> = node.children().collect();
    let mut children = Vec::new();
    let mut past_opening_tag = false;
    let mut index = 0;
    while index < node_children.len() {
        let child = node_children[index];
        match child {
            RedTree::Node(child_node) => {
                if child_node.element_type() == HtmlElementType::TagSlashOpen && closes_current_element(&node_children, index, tag_name.as_deref(), source) {
                    break;
                }
                if past_opening_tag {
                    children.push(child);
                }
            }
            RedTree::Leaf(leaf) => {
                if matches!(leaf.kind, HtmlTokenType::TagClose | HtmlTokenType::TagSelfClose) {
                    if !past_opening_tag {
                        past_opening_tag = true;
                    }
                }
                else if leaf.kind == HtmlTokenType::TagSlashOpen {
                    if closes_current_element(&node_children, index, tag_name.as_deref(), source) {
                        break;
                    }
                }
                else if past_opening_tag {
                    children.push(child);
                }
            }
        }
        index += 1;
    }
    children
}

fn closes_current_element<'a>(children: &[RedTree<'a, HtmlLanguage>], slash_index: usize, tag_name: Option<&str>, source: &SourceText) -> bool {
    let Some(expected) = tag_name
    else {
        return true;
    };
    closing_tag_name_at(children, slash_index, source).is_some_and(|name| name == expected)
}

fn closing_tag_name_at<'a>(children: &[RedTree<'a, HtmlLanguage>], slash_index: usize, source: &SourceText) -> Option<String> {
    for child in children.iter().skip(slash_index + 1) {
        match child {
            RedTree::Leaf(leaf) if leaf.kind == HtmlTokenType::TagName => {
                return Some(leaf.text(source).trim().to_ascii_lowercase());
            }
            RedTree::Node(node) if node.element_type() == HtmlElementType::TagName => {
                return Some(node.text(source).trim().to_ascii_lowercase());
            }
            RedTree::Leaf(leaf) if !HtmlTokenType::is_ignored(&leaf.kind) => return None,
            RedTree::Node(node) if node.element_type() != HtmlElementType::TagName && node.element_type() != HtmlElementType::TagSlashOpen => {
                return None;
            }
            _ => {}
        }
    }
    None
}

fn element_tag_name(node: RedNode<HtmlLanguage>, source: &SourceText) -> Option<String> {
    for child in node.children() {
        match child {
            RedTree::Node(child_node) if child_node.element_type() == HtmlElementType::TagName => {
                let name = child_node.text(source).trim().to_ascii_lowercase();
                if !name.is_empty() {
                    return Some(name);
                }
            }
            RedTree::Leaf(leaf) if leaf.kind == HtmlTokenType::TagName => {
                let name = leaf.text(source).trim().to_ascii_lowercase();
                if !name.is_empty() {
                    return Some(name);
                }
            }
            _ => {}
        }
    }
    tag_name_from_element_text(node.text(source).as_ref())
}

fn tag_name_from_element_text(text: &str) -> Option<String> {
    let trimmed = text.trim_start();
    if !trimmed.starts_with('<') || trimmed.starts_with("</") {
        return None;
    }
    let inner = &trimmed[1..];
    let end = inner.find(|ch: char| ch.is_whitespace() || ch == '>' || ch == '/').unwrap_or(inner.len());
    let tag = inner[..end].trim().to_ascii_lowercase();
    if tag.is_empty() { None } else { Some(tag) }
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"')) || (trimmed.starts_with('\'') && trimmed.ends_with('\'')) { trimmed[1..trimmed.len() - 1].to_string() } else { trimmed.to_string() }
}

fn normalize_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{HtmlBuilder, HtmlLanguage, HtmlParser};
    use oak_core::{Builder, Parser, SourceText, parser::session::ParseSession};

    #[test]
    fn builder_lowers_nested_elements_and_attributes() {
        const SOURCE: &str = r#"<article class="doc"><h1>Title</h1><p>Body</p></article>"#;
        let language = HtmlLanguage::default();
        let builder = HtmlBuilder::new(language);
        let source = SourceText::new(SOURCE);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let built = builder.build(&source, &[], &mut cache);
        let document = built.result.expect("build html ast");
        assert_eq!(document.nodes.len(), 1);
        let HtmlNode::Element(article) = &document.nodes[0]
        else {
            panic!("expected article element");
        };
        assert_eq!(article.tag_name, "article");
        assert_eq!(article.attributes.iter().find(|attr| attr.name == "class").and_then(|attr| attr.value.as_deref()), Some("doc"));
        assert_eq!(article.children.len(), 2);
    }

    #[test]
    fn builder_lowers_nav_links_for_selector_execution() {
        const SOURCE: &str = r#"<nav epub:type="toc"><ol><li><a href="part.xhtml">Part</a></li></ol></nav>"#;
        let language = HtmlLanguage::default();
        let source = SourceText::new(SOURCE);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let parsed = HtmlParser::new(&language).parse(&source, &[], &mut cache);
        let green = parsed.result.expect("parse html");
        let document = build_document(green, &source);
        let view = crate::query::HtmlDocumentView::from_document(&document);
        let matched = crate::query::select_css(&view, "nav a[href]", oak_core::query::QueryBudget::default()).expect("css selector");
        assert_eq!(matched.matches.len(), 1);
    }

    #[test]
    fn lexes_nested_nav_closing_tags() {
        use crate::{HtmlLanguage, HtmlLexer};
        use oak_core::{Lexer, Source};
        const SOURCE: &str = r#"<li><a href="part.xhtml">Part One</a>
    <ol>
      <li><a href="part.xhtml#ch1">Chapter 1</a></li>
    </ol>
  </li>"#;
        let language = HtmlLanguage::default();
        let source = SourceText::new(SOURCE);
        let lexer = HtmlLexer::new(&language);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let tokens = lexer.lex(&source, &[], &mut cache).result.expect("lex");
        let rendered = tokens.iter().map(|token| format!("{:?}:{}", token.kind, source.get_text_in(token.span.clone()))).collect::<Vec<_>>().join("\n");
        assert!(rendered.contains("TagSlashOpen:</"), "missing closing tags in token stream:\n{rendered}");
        assert!(rendered.contains("Text:Part One"), "missing anchor text token:\n{rendered}");
    }

    #[test]
    fn parser_builds_nested_nav_green_tree() {
        use crate::HtmlParser;
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
        let document = build_document(green, &source);
        let HtmlNode::Element(nav) = &document.nodes[0]
        else {
            panic!("expected nav root element");
        };
        fn count_tag(element: &Element, tag: &str) -> usize {
            let mut count = if element.tag_name.eq_ignore_ascii_case(tag) { 1 } else { 0 };
            for child in &element.children {
                if let HtmlNode::Element(child_element) = child {
                    count += count_tag(child_element, tag);
                }
            }
            count
        }
        assert_eq!(nav.tag_name, "nav");
        assert_eq!(count_tag(nav, "a"), 2);
    }

    #[test]
    fn builder_lowers_html_body_section_paragraph() {
        const SOURCE: &str = r#"<html><body><section><div><p>Wrapped</p></div></section></body></html>"#;
        let language = HtmlLanguage::default();
        let source = SourceText::new(SOURCE);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let built = HtmlBuilder::new(language).build(&source, &[], &mut cache);
        let document = built.result.expect("build html ast");
        fn find_p_text(element: &Element) -> Option<String> {
            if element.tag_name.eq_ignore_ascii_case("p") {
                for child in &element.children {
                    if let HtmlNode::Text(text) = child {
                        return Some(text.content.clone());
                    }
                }
            }
            for child in &element.children {
                if let HtmlNode::Element(child_element) = child {
                    if let Some(found) = find_p_text(child_element) {
                        return Some(found);
                    }
                }
            }
            None
        }
        let text = document
            .nodes
            .iter()
            .filter_map(|node| match node {
                HtmlNode::Element(element) => find_p_text(element),
                _ => None,
            })
            .next()
            .expect("paragraph text");
        assert_eq!(text, "Wrapped");
    }

    #[test]
    fn builder_lowers_nested_nav_anchor_text() {
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
        let built = HtmlBuilder::new(language).build(&source, &[], &mut cache);
        let document = built.result.expect("build html ast");
        fn count_tag(element: &Element, tag: &str) -> usize {
            let mut count = if element.tag_name.eq_ignore_ascii_case(tag) { 1 } else { 0 };
            for child in &element.children {
                if let HtmlNode::Element(child_element) = child {
                    count += count_tag(child_element, tag);
                }
            }
            count
        }
        let HtmlNode::Element(nav) = &document.nodes[0]
        else {
            panic!("expected nav root");
        };
        fn dump(element: &Element, depth: usize) -> String {
            let mut out = format!("{}{}\n", "  ".repeat(depth), element.tag_name);
            for child in &element.children {
                match child {
                    HtmlNode::Element(child_element) => out.push_str(&dump(child_element, depth + 1)),
                    HtmlNode::Text(text) => out.push_str(&format!("{}\"{}\"\n", "  ".repeat(depth + 1), text.content)),
                    HtmlNode::Comment(_) => {}
                }
            }
            out
        }
        let tree_dump = dump(nav, 0);
        assert_eq!(count_tag(nav, "a"), 2, "expected two anchors in nav tree:\n{tree_dump}");
        let view = crate::query::HtmlDocumentView::from_document(&document);
        let (_, anchors) = crate::query::select_css_elements(&view, "nav a[href]", oak_core::query::QueryBudget::default()).expect("css selector");
        assert_eq!(anchors.len(), 2);
        let labels = anchors
            .iter()
            .map(|anchor| {
                anchor
                    .children
                    .iter()
                    .filter_map(|child| match child {
                        HtmlNode::Text(text) => Some(text.content.as_str()),
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join("")
            })
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["Part One", "Chapter 1"]);
    }
    #[test]
    fn lexes_text_content_outside_tags() {
        use crate::{HtmlLanguage, HtmlLexer, lexer::token_type::HtmlTokenType};
        use oak_core::{Lexer, Source, SourceText, parser::session::ParseSession};
        const SOURCE: &str = r#"<a href="part.xhtml">Part One</a>"#;
        let language = HtmlLanguage::default();
        let source = SourceText::new(SOURCE);
        let lexer = HtmlLexer::new(&language);
        let mut cache = ParseSession::<HtmlLanguage>::default();
        let lexed = lexer.lex(&source, &[], &mut cache);
        let tokens = lexed.result.expect("lex");
        let text = tokens.iter().find(|token| token.kind == HtmlTokenType::Text).map(|token| source.get_text_in(token.span.clone()).to_string());
        assert_eq!(text.as_deref(), Some("Part One"));
    }
}
