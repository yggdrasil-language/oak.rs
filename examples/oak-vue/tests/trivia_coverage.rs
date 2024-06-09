//! Node spans end at their last token, and the root still covers the whole source.

use oak_core::{Builder, GreenNode, GreenTree, ParseSession, Parser, Source, SourceText};
use oak_vue::{
    VueBuilder, VueLanguage, VueParser,
    ast::{VueAttribute, VueNode},
};

/// Walks the tree in source order and returns the offset one past the last covered byte.
fn covered(node: &GreenNode<'_, VueLanguage>, start: usize) -> usize {
    let mut current = start;
    for child in node.children {
        current = match child {
            GreenTree::Node(child) => covered(child, current),
            GreenTree::Leaf(leaf) => current + leaf.length as usize,
        };
    }
    current
}

#[test]
fn shorthand_event_directive_name_is_the_identifier_only() {
    let source = SourceText::new(r#"<template><Like client:idle @click="armed = !armed" /></template>"#);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    assert!(parser.parse(&source, &[], &mut session).result.is_ok());

    let mut session = ParseSession::default();
    let built = VueBuilder::new().build(&source, &[], &mut session).result.unwrap();
    let VueNode::Element(element) = &built.blocks[0].children[0]
    else {
        panic!("expected component")
    };

    let VueAttribute::Directive(client) = &element.attributes[0]
    else {
        panic!("expected client directive")
    };
    assert_eq!(source.get_text_in(client.name.clone()), "client");
    assert_eq!(source.get_text_in(client.arg.as_ref().unwrap().span.clone()), "idle");

    // The `@` prefix belongs to the tag, so the name span starts at `c`. A directive
    // span that ended one byte late would shift it onto the preceding whitespace.
    let VueAttribute::Directive(click) = &element.attributes[1]
    else {
        panic!("expected click directive")
    };
    assert_eq!(source.get_text_in(click.name.clone()), "click");
    assert!(click.arg.is_none());
}

#[test]
fn root_covers_a_multi_element_template_document() {
    // Every node ends at its last token, so a dropped whitespace inside an expression
    // would shift every following span and leave the root short of the source end.
    let source = SourceText::new(r#"<template><button @click="(() => armed = !armed)" :title="label">{{ label }}</button><li v-for="item in items" :key="item.id">{{ item }}</li></template>"#);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let root = parser.parse(&source, &[], &mut session).result.expect("template CST");

    assert_eq!(covered(root, 0), source.length());
    assert_eq!(root.byte_length as usize, source.length());

    // The trailing newline stays at the root, so the document still covers all of it.
    let source = SourceText::new(
        r#"<template><button @click="(() => armed = !armed)">{{ label }}</button></template>
"#,
    );
    let mut session = ParseSession::default();
    let root = parser.parse(&source, &[], &mut session).result.expect("template CST");
    assert_eq!(covered(root, 0), source.length());
    assert_eq!(root.byte_length as usize, source.length());

    // A node must not take the trivia that follows its last token.
    let template_tag = root
        .children
        .iter()
        .find_map(|child| match child {
            GreenTree::Node(node) => match node.children.first() {
                Some(GreenTree::Node(tag)) => Some(tag),
                _ => None,
            },
            _ => None,
        })
        .expect("template start tag");
    assert_eq!(covered(template_tag, 0), "<template>".len());
}
