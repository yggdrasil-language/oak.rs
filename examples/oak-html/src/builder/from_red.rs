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
        nodes.extend(build_top_level_node(child, source));
    }
    HtmlDocument { nodes }
}

fn build_top_level_node(tree: RedTree<HtmlLanguage>, source: &SourceText) -> Vec<HtmlNode> {
    match tree {
        RedTree::Node(node) if node.element_type() == HtmlElementType::Element => flatten_element(build_element_node(node, source)),
        _ => Vec::new(),
    }
}

fn flatten_element(built: Option<(HtmlNode, Vec<HtmlNode>)>) -> Vec<HtmlNode> {
    let Some((element, mut trailing)) = built
    else {
        return Vec::new();
    };
    let mut nodes = vec![element];
    nodes.append(&mut trailing);
    nodes
}

fn build_element_node(node: RedNode<HtmlLanguage>, source: &SourceText) -> Option<(HtmlNode, Vec<HtmlNode>)> {
    let tag_name = element_tag_name(node, source)?;
    let attributes = collect_attributes(node, source);
    let element_children = element_body_children(node, source);
    let mut children = Vec::new();
    for child in element_children.body {
        children.extend(build_element_child(child, source));
    }
    let mut trailing = Vec::new();
    for child in element_children.trailing {
        trailing.extend(build_element_child(child, source));
    }
    let span = core::range::Range { start: node.span().start, end: element_children.closing_end };
    Some((HtmlNode::Element(Element { tag_name, attributes, children, span }), trailing))
}

fn build_element_child(tree: RedTree<HtmlLanguage>, source: &SourceText) -> Vec<HtmlNode> {
    match tree {
        RedTree::Node(node) if node.element_type() == HtmlElementType::Element => flatten_element(build_element_node(node, source)),
        RedTree::Node(node) if node.element_type() == HtmlElementType::Text => {
            let content = normalize_whitespace(node.text(source).as_ref());
            if content.is_empty() { Vec::new() } else { vec![HtmlNode::Text(Text { content, span: node.span() })] }
        }
        RedTree::Leaf(leaf) => match leaf.kind {
            HtmlTokenType::Text | HtmlTokenType::Whitespace | HtmlTokenType::Newline => {
                let content = normalize_whitespace(leaf.text(source).as_ref());
                if content.is_empty() { Vec::new() } else { vec![HtmlNode::Text(Text { content, span: leaf.span() })] }
            }
            HtmlTokenType::EntityRef | HtmlTokenType::CharRef => {
                let raw = leaf.text(source).into_owned();
                let content = decode_character_reference(&raw).unwrap_or(raw);
                if content.is_empty() { Vec::new() } else { vec![HtmlNode::Text(Text { content, span: leaf.span() })] }
            }
            _ => Vec::new(),
        },
        _ => Vec::new(),
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

/// Children of an element split into the element's own content and the trivia
/// that the parser attached after its closing tag.
///
/// `skip_trivia` pushes ignored tokens into the node that is still open, so a
/// space after `</b>` lands inside the `b` element. It belongs to the parent
/// text flow instead, and `closing_end` keeps the element span exact.
struct ElementChildren<'a> {
    body: Vec<RedTree<'a, HtmlLanguage>>,
    trailing: Vec<RedTree<'a, HtmlLanguage>>,
    closing_end: usize,
}

fn element_body_children<'a>(node: RedNode<'a, HtmlLanguage>, source: &SourceText) -> ElementChildren<'a> {
    let tag_name = element_tag_name(node, source);
    let node_children: Vec<RedTree<'a, HtmlLanguage>> = node.children().collect();
    let mut body = Vec::new();
    let mut trailing = Vec::new();
    let mut closing_end = None;
    let mut past_opening_tag = false;
    let mut index = 0;
    while index < node_children.len() {
        let child = node_children[index];
        match child {
            RedTree::Node(child_node) => {
                if child_node.element_type() == HtmlElementType::TagSlashOpen && closes_current_element(&node_children, index, tag_name.as_deref(), source) {
                    if let Some(close_index) = closing_tag_close_index(&node_children, index) {
                        closing_end = Some(tree_span_end(node_children[close_index]));
                        trailing.extend(node_children[close_index + 1..].iter().copied());
                    }
                    break;
                }
                if past_opening_tag {
                    body.push(child);
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
                        if let Some(close_index) = closing_tag_close_index(&node_children, index) {
                            closing_end = Some(tree_span_end(node_children[close_index]));
                            trailing.extend(node_children[close_index + 1..].iter().copied());
                        }
                        break;
                    }
                }
                else if past_opening_tag {
                    body.push(child);
                }
            }
        }
        index += 1;
    }
    ElementChildren { body, trailing, closing_end: closing_end.unwrap_or_else(|| node.span().end) }
}

/// Index of the `>` token that terminates the closing tag starting at `slash_index`.
fn closing_tag_close_index(children: &[RedTree<'_, HtmlLanguage>], slash_index: usize) -> Option<usize> {
    children.iter().enumerate().skip(slash_index + 1).find_map(|(index, child)| match child {
        RedTree::Leaf(leaf) if leaf.kind == HtmlTokenType::TagClose => Some(index),
        _ => None,
    })
}

fn tree_span_end(tree: RedTree<'_, HtmlLanguage>) -> usize {
    match tree {
        RedTree::Node(node) => node.span().end,
        RedTree::Leaf(leaf) => leaf.span().end,
    }
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
    let collapsed = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if collapsed.is_empty() {
        return if text.is_empty() { String::new() } else { " ".to_string() };
    }
    let mut normalized = String::new();
    if text.starts_with(char::is_whitespace) {
        normalized.push(' ');
    }
    normalized.push_str(&collapsed);
    if text.ends_with(char::is_whitespace) {
        normalized.push(' ');
    }
    normalized
}

/// Decode an HTML character reference token into the character it denotes.
///
/// Unknown named references stay verbatim so the syntax layer never invents
/// text that was not authored.
fn decode_character_reference(raw: &str) -> Option<String> {
    let body = raw.strip_prefix('&')?.strip_suffix(';')?;
    if let Some(digits) = body.strip_prefix('#') {
        let code = match digits.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok()?,
            None => digits.parse::<u32>().ok()?,
        };
        return char::from_u32(code).map(|character| character.to_string());
    }
    let character = match body {
        "amp" => '&',
        "lt" => '<',
        "gt" => '>',
        "quot" => '"',
        "apos" => '\'',
        "nbsp" => '\u{00a0}',
        _ => return None,
    };
    Some(character.to_string())
}
