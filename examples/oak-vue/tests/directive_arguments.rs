use oak_core::{Builder, ParseSession, Parser, Source, SourceText};
use oak_vue::{VueAttribute, VueBuilder, VueLanguage, VueNode, VueParser};

#[test]
fn builder_preserves_directive_arguments() {
    let source = SourceText::new(r#"<template><Like client:idle @click="armed = !armed" /></template>"#);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut parse_session = ParseSession::default();
    assert!(parser.parse(&source, &[], &mut parse_session).result.is_ok());
    let builder = VueBuilder::new();
    let mut build_session = ParseSession::default();
    let root = builder.build(&source, &[], &mut build_session).result.unwrap();
    let VueNode::Element(element) = &root.blocks[0].children[0] else { panic!("expected component") };
    let VueAttribute::Directive(client) = &element.attributes[0] else { panic!("expected client directive") };
    assert_eq!(source.get_text_in(client.name.clone()), "client");
    assert_eq!(source.get_text_in(client.arg.as_ref().unwrap().span.clone()), "idle");
    let VueAttribute::Directive(click) = &element.attributes[1] else { panic!("expected click directive") };
    assert_eq!(source.get_text_in(click.name.clone()), "click");
    assert!(click.arg.is_none());
}
