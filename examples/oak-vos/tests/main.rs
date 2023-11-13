use oak_vos::{VosDeclarationKind, VosSyntaxElement, VosTokenType, parse};

#[test]
fn parses_vos_schema_through_oak() {
    let root = parse(
        "namespace demo::identity\n table User { @@id: uuid, embedding: vec<3>, asset: file, }",
    )
    .expect("Oak parses the VOS surface");

    assert!(root.source.contains("table User"));
    assert_eq!(root.declarations.iter().filter(|item| item.kind == VosDeclarationKind::Table).count(), 1);
    assert_eq!(root.declarations.iter().find(|item| item.kind == VosDeclarationKind::Table).and_then(|item| item.name.as_deref()), Some("User"));
    assert_eq!(root.declarations.iter().find(|item| item.kind == VosDeclarationKind::Namespace).and_then(|item| item.path.as_deref()), Some(["demo".to_owned(), "identity".to_owned()].as_slice()));
}

#[test]
fn preserves_current_vos_micro_authoring_surface() {
    let source = "micro normalize_name(value: utf8) -> utf8 { let name = value.trim() name.lower() }";
    let root = parse(source).expect("Oak preserves the current micro surface");
    assert_eq!(root.source, source);
    assert_eq!(root.declarations[0].kind, VosDeclarationKind::Micro);
}

#[test]
fn classifies_query_and_udf_declarations() {
    let root = parse(
        "query active_users() { table User } udf normalize(value: utf8) -> utf8 { value }",
    )
    .expect("Oak recognizes VOS query and udf declarations");

    assert_eq!(root.declarations[0].kind, VosDeclarationKind::Query);
    assert_eq!(root.declarations[0].name.as_deref(), Some("active_users"));
    assert_eq!(root.declarations[1].kind, VosDeclarationKind::Udf);
    assert_eq!(root.declarations[1].name.as_deref(), Some("normalize"));
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

#[test]
fn obsolete_table_does_not_become_a_table_declaration() {
    let root = parse("obsolete table Dealer;").expect("Oak parses obsolete table declarations");
    assert_eq!(root.declarations.len(), 1);
    assert_eq!(root.declarations[0].kind, VosDeclarationKind::Obsolete);
    assert_eq!(root.declarations[0].name.as_deref(), Some("Dealer"));
}

#[test]
fn builder_exposes_lossless_cst_tokens_and_spans() {
    let source = "using shared::UserId;\ntable User { @@id: uuid, }";
    let root = parse(source).expect("Oak parses the VOS surface");
    assert_eq!(root.syntax.span, (0..source.len()).into());
    let using = root.syntax.children.iter().find_map(|element| match element {
        VosSyntaxElement::Node(node) if node.kind == oak_vos::VosElementType::Using => Some(node),
        _ => None,
    }).expect("using CST node");
    assert_eq!(&source[using.span.clone()], "using shared::UserId;\n");
    assert!(using.children.iter().any(|element| matches!(
        element,
        VosSyntaxElement::Token(token) if token.kind == VosTokenType::Identifier && token.text == "shared"
    )));
}

#[test]
fn builder_exposes_using_path_without_reparsing_tokens() {
    let root = parse("using shared::UserId;").expect("Oak parses using");
    assert_eq!(root.declarations[0].kind, VosDeclarationKind::Using);
    assert_eq!(root.declarations[0].path.as_deref(), Some(["shared".to_owned(), "UserId".to_owned()].as_slice()));
}
