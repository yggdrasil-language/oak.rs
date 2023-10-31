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
    let VueNode::Element(element) = &root.blocks[0].children[0]
    else {
        panic!("expected component")
    };
    let VueAttribute::Directive(client) = &element.attributes[0]
    else {
        panic!("expected client directive")
    };
    assert_eq!(source.get_text_in(client.name.clone()), "client");
    assert_eq!(source.get_text_in(client.arg.as_ref().unwrap().span.clone()), "idle");
    let VueAttribute::Directive(click) = &element.attributes[1]
    else {
        panic!("expected click directive")
    };
    assert_eq!(source.get_text_in(click.name.clone()), "click");
    assert!(click.arg.is_none());
}

#[test]
fn valueless_directive_does_not_consume_next_attribute() {
    let source = SourceText::new(r#"<template><button v-else :type="type" disabled @click="run" /></template>"#);
    let mut session = ParseSession::default();
    let root = VueBuilder::new().build(&source, &[], &mut session).result.unwrap();
    let VueNode::Element(element) = &root.blocks[0].children[0]
    else {
        panic!("expected button")
    };
    assert_eq!(element.attributes.len(), 4);
    let VueAttribute::Directive(otherwise) = &element.attributes[0]
    else {
        panic!("expected else")
    };
    assert_eq!(source.get_text_in(otherwise.span.clone()).trim(), "v-else");
    assert!(otherwise.arg.is_none());
    assert!(otherwise.value.is_none());
    let VueAttribute::Directive(binding) = &element.attributes[1]
    else {
        panic!("expected binding")
    };
    assert_eq!(source.get_text_in(binding.span.clone()).trim(), r#":type="type""#);
}
