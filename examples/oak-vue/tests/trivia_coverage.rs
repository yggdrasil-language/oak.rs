//! Node spans end at their last token, and the root still covers the whole source.

use oak_core::{Builder, ParseSession, Parser, Source, SourceText};
use oak_vue::{
    VueBuilder, VueLanguage, VueParser,
    ast::{VueAttribute, VueNode},
};

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
