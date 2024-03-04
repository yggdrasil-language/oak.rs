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
