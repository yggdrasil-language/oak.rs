use oak_core::{Builder, ParseSession, SourceText};
use oak_vue::{VueBuilder, VueNode};

fn build(src: &str) -> usize {
    let source = SourceText::new(src);
    let mut session = ParseSession::default();
    let built = Builder::build(&VueBuilder::new(), &source, &[], &mut session);
    let total = built.diagnostics.len() + usize::from(built.result.is_err());
    for d in &built.diagnostics {
        println!("{d:?}");
    }
    total
}

fn first_element_attr_count(src: &str) -> usize {
    let source = SourceText::new(src);
    let mut session = ParseSession::default();
    let built = Builder::build(&VueBuilder::new(), &source, &[], &mut session);
    let root = built.result.expect("root");
    let block = root.blocks.first().expect("template block");
    let element = block
        .children
        .iter()
        .find_map(|node| match node {
            VueNode::Element(element) => Some(element),
            _ => None,
        })
        .expect("element");
    element.attributes.len()
}

#[test]
fn unquoted_attribute_value_does_not_swallow_the_closing_tag() {
    // Vue treats an unquoted plain-attribute value as static text.
    assert_eq!(build("<template><p title={a}>x</p></template>"), 0);
    assert_eq!(build("<template><p if={ok}>{label}</p></template>"), 0);
    assert_eq!(build("<template><p title={a} id={b}>x</p></template>"), 0);
}

#[test]
fn unquoted_attribute_value_stops_at_whitespace() {
    assert_eq!(build("<template><p title=a id=b>x</p></template>"), 0);
}

#[test]
fn quoted_attributes_stay_separate() {
    assert_eq!(first_element_attr_count(r#"<template><p title="a" title="b">x</p></template>"#), 2);
    assert_eq!(first_element_attr_count("<template><p title=a id=b>x</p></template>"), 2);
}
