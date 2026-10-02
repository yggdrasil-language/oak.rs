use oak_vos::parse;

#[test]
fn parses_vos_schema_through_oak() {
    let root = parse(
        "namespace demo::identity\n table User { @@id: uuid, embedding: vec<3>, asset: file, }",
    )
    .expect("Oak parses the VOS surface");

    assert!(root.source.contains("table User"));
}

#[test]
fn preserves_current_vos_micro_authoring_surface() {
    let source = "micro normalize_name(value: utf8) -> utf8 { let name = value.trim() name.lower() }";
    let root = parse(source).expect("Oak preserves the current micro surface");
    assert_eq!(root.source, source);
}

#[test]
fn rejects_lexical_errors_before_vos_semantic_lowering() {
    let error = parse("table User { name: utf8; \0 }").expect_err("Oak rejects NUL");
    assert!(!error.is_empty());
}

#[test]
fn rejects_unterminated_string() {
    assert!(parse("table User { name: utf8 = \"unterminated }").is_err());
}

#[test]
fn rejects_mismatched_and_missing_delimiters() {
    for source in ["table User { name: utf8", "table User { name: [utf8) }", "User.filter(x => x.name]", "}"] {
        assert!(parse(source).is_err(), "accepted malformed source: {source}");
    }
}

#[test]
fn namespace_does_not_swallow_following_declarations() {
    use oak_core::{Parser, SourceText, ParseSession, GreenTree};
    use oak_vos::{VosLanguage, VosParser, VosElementType};
    let language = VosLanguage;
    let source = SourceText::new("namespace demo::identity\n table User { @@id: uuid, }");
    let mut cache = ParseSession::<VosLanguage>::default();
    let result = VosParser::new(&language).parse(&source, &[], &mut cache);
    let root = result.result.unwrap();
    assert_eq!(root.byte_length as usize, source.text().len());
    assert!(root.children.iter().any(|child| matches!(child, GreenTree::Node(node) if node.kind == VosElementType::Table)));
}
