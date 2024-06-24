use oak_vos::{VosBinaryOp, VosDeclarationKind, VosExpr, VosLiteral, VosPathSep, VosProjItem, VosStmt, VosSyntaxElement, VosTokenType, VosTypeArgument, VosTypeSyntax, parse, parse_program};

#[test]
fn parses_vos_schema_through_oak() {
    let root = parse("namespace demo::identity\n table User { @@id: uuid, embedding: vec<3>, asset: file, }").expect("Oak parses the VOS surface");

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
fn preserves_vos_macro_authoring_surface() {
    let root = parse("macro public_name(value: utf8) -> utf8 { value.trim() }").expect("Oak parses macro declarations");
    assert_eq!(root.declarations[0].kind, VosDeclarationKind::Macro);
    assert_eq!(root.declarations[0].name.as_deref(), Some("public_name"));
    assert_eq!(root.declarations[0].parameters[0].name, "value");
    assert_eq!(root.declarations[0].return_type.as_ref().unwrap().text, "-> utf8");
}

#[test]
fn classifies_query_and_udf_declarations() {
    let root = parse("query active_users() { table User } udf normalize(value: utf8) -> utf8 { value }").expect("Oak recognizes VOS query and udf declarations");

    assert_eq!(root.declarations[0].kind, VosDeclarationKind::Query);
    assert_eq!(root.declarations[0].name.as_deref(), Some("active_users"));
    assert_eq!(root.declarations[1].kind, VosDeclarationKind::Udf);
    assert_eq!(root.declarations[1].name.as_deref(), Some("normalize"));
    assert_eq!(root.declarations[0].signature.as_ref().unwrap().text, "()");
    assert_eq!(root.declarations[0].body.as_ref().unwrap().text, "{ table User }");
    assert_eq!(root.declarations[1].signature.as_ref().unwrap().text, "(value: utf8)");
    assert_eq!(root.declarations[1].parameters[0].name, "value");
    assert!(matches!(&root.declarations[1].parameters[0].type_expr, VosTypeSyntax::Named { path, .. } if path == &["utf8".to_owned()]));
    assert_eq!(root.declarations[1].body.as_ref().unwrap().text, "{ value }");
    assert_eq!(root.declarations[1].return_type.as_ref().unwrap().text, "-> utf8");
    assert!(matches!(
        root.declarations[1].return_type_expr.as_ref(),
        Some(VosTypeSyntax::Named { path, .. }) if path == &["utf8".to_owned()]
    ));
}

#[test]
fn operation_declaration_slices_keep_exact_source_spans() {
    let source = "# keep trivia\nquery active_users( status: utf8 ) {\n  User.filter(x => x.status == status)\n}\n\nmicro trim(value: utf8) -> utf8 { value.trim() }";
    let root = parse(source).expect("Oak parses operation declarations");

    let query = &root.declarations[0];
    let query_signature = query.signature.as_ref().expect("query signature");
    let query_body = query.body.as_ref().expect("query body");
    assert_eq!(&source[query_signature.span.clone()], query_signature.text);
    assert_eq!(&source[query_body.span.clone()], query_body.text);
    assert_eq!(query_signature.text, "( status: utf8 )");
    assert_eq!(query_body.text, "{\n  User.filter(x => x.status == status)\n}");

    let micro = &root.declarations[1];
    let micro_signature = micro.signature.as_ref().expect("micro signature");
    let micro_body = micro.body.as_ref().expect("micro body");
    let micro_return = micro.return_type.as_ref().expect("micro return type");
    assert_eq!(&source[micro_signature.span.clone()], micro_signature.text);
    assert_eq!(&source[micro_body.span.clone()], micro_body.text);
    assert_eq!(&source[micro_return.span.clone()], micro_return.text);
    assert_eq!(micro_signature.text, "(value: utf8)");
    assert_eq!(micro_return.text, "-> utf8");
    assert!(matches!(
        micro.return_type_expr.as_ref(),
        Some(VosTypeSyntax::Named { path, .. }) if path == &["utf8".to_owned()]
    ));
    assert_eq!(micro_body.text, "{ value.trim() }");
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
fn rejects_malformed_operation_parameters() {
    for source in ["query missing_colon(value utf8) { value }", "query missing_type(value:) { value }", "query missing_comma(first: utf8 second: utf8) { first }"] {
        assert!(parse(source).is_err(), "accepted malformed operation: {source}");
    }
}

#[test]
fn rejects_mismatched_and_missing_delimiters() {
    for source in ["table User { name: utf8", "table User { name: [utf8) }", "User.filter(x => x.name]", "}"] {
        assert!(parse(source).is_err(), "accepted malformed source: {source}");
    }
}

#[test]
fn namespace_does_not_swallow_following_declarations() {
    use oak_core::{GreenTree, ParseSession, Parser, SourceText};
    use oak_vos::{VosElementType, VosLanguage, VosParser};
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
    let using = root
        .syntax
        .children
        .iter()
        .find_map(|element| match element {
            VosSyntaxElement::Node(node) if node.kind == oak_vos::VosElementType::Using => Some(node),
            _ => None,
        })
        .expect("using CST node");
    // A node spans its own tokens only, so the following newline stays outside it.
    assert_eq!(&source[using.span.clone()], "using shared::UserId;");
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

#[test]
fn builder_projects_fields_with_exact_syntax_spans() {
    let source = "table User { @@id: uuid, [unique] email: utf8 = \"anonymous\", manager: &shared::User? = null, tags: [utf8]?, embedding: vec<3>, asset: file, }";
    let root = parse(source).expect("Oak builds field syntax");
    let fields = &root.declarations[0].fields;
    assert_eq!(fields.iter().map(|field| field.name.as_str()).collect::<Vec<_>>(), ["id", "email", "manager", "tags", "embedding", "asset"]);
    assert_eq!(fields[0].attributes[0].text, "@@");
    assert_eq!(fields[1].attributes[0].text, "[unique]");
    assert_eq!(fields[1].attributes[0].name.as_deref(), Some("unique"));
    assert_eq!(fields[1].default_value.as_ref().unwrap().text, "\"anonymous\"");
    assert_eq!(fields[2].type_syntax.text, "&shared::User?");
    assert_eq!(fields[2].default_value.as_ref().unwrap().text, "null");
    assert_eq!(fields[3].type_syntax.text, "[utf8]?");
    assert_eq!(fields[4].type_syntax.text, "vec<3>");
    assert_eq!(fields[5].type_syntax.text, "file");
    for field in fields {
        assert_eq!(&source[field.name_span.clone()], field.name);
        assert_eq!(&source[field.type_syntax.span.clone()], field.type_syntax.text);
        assert!(field.span.start <= field.name_span.start && field.span.end >= field.type_syntax.span.end);
        for attribute in &field.attributes {
            assert_eq!(&source[attribute.span.clone()], attribute.text);
        }
        if let Some(default) = &field.default_value {
            assert_eq!(&source[default.span.clone()], default.text);
        }
    }
}

#[test]
fn builder_preserves_attribute_groups_and_custom_arguments() {
    let source = "table Account { [primary, wire] [backfill(\"legacy, value\")] id: utf8 = \"\", @email: utf8 }";
    let root = parse(source).unwrap();
    let fields = &root.declarations[0].fields;
    assert_eq!(fields[0].attributes.len(), 2);
    assert_eq!(fields[0].attributes[0].text, "[primary, wire]");
    assert_eq!(fields[0].attributes[1].text, "[backfill(\"legacy, value\")]");
    assert_eq!(fields[1].attributes[0].text, "@");
}

#[test]
fn attribute_spans_exclude_following_comments() {
    let source = "table Account { [primary] # key constraint\n id: uuid, @email: utf8 }";
    let root = parse(source).unwrap();
    let attribute = &root.declarations[0].fields[0].attributes[0];
    assert_eq!(attribute.text, "[primary]");
    assert_eq!(&source[attribute.span.clone()], "[primary]");
    assert_eq!(root.declarations[0].fields[1].attributes[0].text, "@");
}

#[test]
fn class_fields_support_newlines_comments_and_nested_types() {
    let source = "class 数据 {\n [primary] 标识: uuid # trailing type comment\n entries: list<&shared::User?>\n values: [[utf8]?]?\n number: i64 = -42\n active: bool = true\n }";
    let root = parse(source).unwrap();
    let fields = &root.declarations[0].fields;
    assert_eq!(fields.len(), 5);
    assert_eq!(fields[0].name, "标识");
    assert_eq!(fields[0].type_syntax.text, "uuid");
    assert_eq!(fields[1].type_syntax.text, "list<&shared::User?>");
    assert_eq!(fields[2].type_syntax.text, "[[utf8]?]?");
    assert_eq!(fields[3].default_value.as_ref().unwrap().text, "-42");
    assert_eq!(fields[4].default_value.as_ref().unwrap().text, "true");
    assert_eq!(&source[fields[0].name_span.clone()], "标识");
}

#[test]
fn rejects_incomplete_field_syntax() {
    for source in ["table T { id uuid }", "table T { id: }", "class T { [primary] : uuid }", "class T { id: utf8 = }", "table T { id: vec<> }", "table T { id: list<utf8 }", "table T { id: & }", "table T { id: uuid = -true }"] {
        assert!(parse(source).is_err(), "accepted incomplete field: {source}");
    }
}

#[test]
fn typed_fields_do_not_change_lossless_cst() {
    fn append_text(node: &oak_vos::VosSyntaxNode, text: &mut String) {
        for child in &node.children {
            match child {
                VosSyntaxElement::Node(inner) => append_text(inner, text),
                VosSyntaxElement::Token(token) => text.push_str(&token.text),
            }
        }
    }
    let source = "# schema\ntable T {\n [primary] id: uuid, # key\n value: [utf8]? = null;\n}\n";
    let root = parse(source).unwrap();
    let mut restored = String::new();
    append_text(&root.syntax, &mut restored);
    assert_eq!(restored, source);
    assert_eq!(root.declarations[0].fields.len(), 2);
}

#[test]
fn builder_projects_structured_type_syntax_without_resolving_names() {
    let source = "table User { id: uuid, manager: &shared::User?, tags: [utf8]?, embedding: vector<3>, rows: list<&User>, instant: DateTime<UTC>, }";
    let root = parse(source).unwrap();
    let fields = &root.declarations[0].fields;
    assert!(matches!(&fields[0].type_expr, VosTypeSyntax::Named { path, .. } if path == &["uuid"]));
    assert!(matches!(&fields[1].type_expr, VosTypeSyntax::Optional { inner, .. } if matches!(inner.as_ref(), VosTypeSyntax::Reference { target, .. } if matches!(target.as_ref(), VosTypeSyntax::Named { path, .. } if path == &["shared", "User"]))));
    assert!(matches!(&fields[2].type_expr, VosTypeSyntax::Optional { inner, .. } if matches!(inner.as_ref(), VosTypeSyntax::List { element, .. } if matches!(element.as_ref(), VosTypeSyntax::Named { path, .. } if path == &["utf8"]))));
    assert!(matches!(&fields[3].type_expr, VosTypeSyntax::Generic { path, arguments, .. } if path == &["vector"] && matches!(&arguments[0], VosTypeArgument::Literal(value) if value.text == "3")));
    assert!(matches!(&fields[4].type_expr, VosTypeSyntax::Generic { path, arguments, .. } if path == &["list"] && matches!(&arguments[0], VosTypeArgument::Type(VosTypeSyntax::Reference { .. }))));
    assert!(matches!(&fields[5].type_expr, VosTypeSyntax::Generic { path, arguments, .. } if path == &["DateTime"] && matches!(&arguments[0], VosTypeArgument::Type(VosTypeSyntax::Named { path, .. }) if path == &["UTC"])));
    for field in fields {
        let span = match &field.type_expr {
            VosTypeSyntax::Named { span, .. } | VosTypeSyntax::Reference { span, .. } | VosTypeSyntax::Optional { span, .. } | VosTypeSyntax::List { span, .. } | VosTypeSyntax::Generic { span, .. } => span,
        };
        assert_eq!(&source[span.clone()], field.type_syntax.text);
    }
}

#[test]
fn structured_type_syntax_rejects_empty_or_malformed_generic_arguments() {
    for source in ["table T { id: vector<> }", "table T { id: list<utf8 }", "table T { id: [utf8 }", "table T { id: & }", "table T { id: shared:: }"] {
        assert!(parse(source).is_err(), "accepted malformed type: {source}");
    }
}

#[test]
fn reference_optional_precedence_preserves_trivia_spans() {
    let source = "class T { owner: &shared::User # reference\n ? }";
    let root = parse(source).unwrap();
    let field = &root.declarations[0].fields[0];
    let VosTypeSyntax::Optional { inner, span } = &field.type_expr
    else {
        panic!("expected optional reference")
    };
    assert_eq!(&source[span.clone()], "&shared::User # reference\n ?");
    let VosTypeSyntax::Reference { target, span } = inner.as_ref()
    else {
        panic!("expected reference")
    };
    assert_eq!(&source[span.clone()], "&shared::User");
    let VosTypeSyntax::Named { path, span } = target.as_ref()
    else {
        panic!("expected named target")
    };
    assert_eq!(path, &["shared", "User"]);
    assert_eq!(&source[span.clone()], "shared::User");
    assert_eq!(field.type_syntax.text, "&shared::User # reference\n ?");
}

#[test]
fn projects_lazy_query_pipeline_program() {
    let source = "let active_users = User.filter(x => x.active && x.user_name != \"\").sort_by(x => x.user_name)\nlet users = active_users.map(x => x.{ user_id, name: user_name }).collect()\n\nusers";
    let program = parse_program(source).expect("Oak projects the operation program");

    assert_eq!(program.statements.len(), 2);
    assert!(matches!(program.result, Some(VosExpr::Name { ref name, .. }) if name == "users"));

    let VosStmt::Let(first) = &program.statements[0]
    else {
        panic!("expected a let statement")
    };
    assert_eq!(first.name, "active_users");
    assert!(first.type_expr.is_none());
    let VosExpr::Call { callee, args, .. } = &first.value
    else {
        panic!("expected a call")
    };
    assert!(matches!(args.len(), 1));
    let VosExpr::Member { object, name, sep, .. } = callee.as_ref()
    else {
        panic!("expected a member callee")
    };
    assert_eq!(name, "sort_by");
    assert_eq!(*sep, VosPathSep::Dot);
    assert!(matches!(object.as_ref(), VosExpr::Call { .. }));

    let VosStmt::Let(second) = &program.statements[1]
    else {
        panic!("expected a let statement")
    };
    let VosExpr::Call { callee, .. } = &second.value
    else {
        panic!("expected a call")
    };
    let VosExpr::Member { object, name, .. } = callee.as_ref()
    else {
        panic!("expected a member callee")
    };
    assert_eq!(name, "collect");
    let VosExpr::Call { args, .. } = object.as_ref()
    else {
        panic!("expected the map call")
    };
    let VosExpr::Lambda(lambda) = &args[0]
    else {
        panic!("expected a lambda argument")
    };
    let VosExpr::StructProj { items, .. } = lambda.body.as_ref()
    else {
        panic!("expected a struct projection")
    };
    assert_eq!(items.len(), 2);
    assert!(matches!(&items[0], VosProjItem::Field(field) if field.name == "user_id" && field.value.is_none()));
    assert!(matches!(&items[1], VosProjItem::Field(field) if field.name == "name" && field.value.is_some()));

    assert_eq!(&source[program.span.clone()], source);
}

#[test]
fn projects_binary_operators_in_precedence_order() {
    let program = parse_program("x.active && x.user_name != \"\"").expect("Oak projects the filter predicate");
    let Some(VosExpr::Binary { op, right, left, .. }) = program.result
    else {
        panic!("expected a binary result")
    };
    assert_eq!(op, VosBinaryOp::And);
    assert!(matches!(left.as_ref(), VosExpr::Member { name, .. } if name == "active"));
    let VosExpr::Binary { op: Ne, right: literal, .. } = right.as_ref()
    else {
        panic!("expected a comparison")
    };
    assert_eq!(*Ne, VosBinaryOp::Ne);
    assert!(matches!(literal.as_ref(), VosExpr::Literal(VosLiteral::String(value)) if value.is_empty()));

    let program = parse_program("a + b * 2 - c").expect("Oak projects arithmetic");
    let Some(VosExpr::Binary { op, .. }) = program.result
    else {
        panic!("expected a binary result")
    };
    assert_eq!(op, VosBinaryOp::Sub);
}

#[test]
fn projects_typed_object_construction_and_write_boundary() {
    let source = "let draft = User { user_id: uuid(), user_name: \"alice\", avatar: null }\nlet stored: User = User { user_id: uuid() }.insert()\nstored";
    let program = parse_program(source).expect("Oak projects construction and insert");

    let VosStmt::Let(draft) = &program.statements[0]
    else {
        panic!("expected a let statement")
    };
    let VosExpr::TypedObject { ty, fields, .. } = &draft.value
    else {
        panic!("expected a typed object")
    };
    assert_eq!(ty, "User");
    assert_eq!(fields.len(), 3);
    assert!(matches!(&fields[0].value, Some(VosExpr::Call { .. })));
    assert!(matches!(&fields[1].value, Some(VosExpr::Literal(VosLiteral::String(value))) if value == "alice"));
    assert!(matches!(&fields[2].value, Some(VosExpr::Literal(VosLiteral::Null))));

    let VosStmt::Let(stored) = &program.statements[1]
    else {
        panic!("expected a let statement")
    };
    assert!(matches!(stored.type_expr, Some(VosTypeSyntax::Named { ref path, .. }) if path == &["User".to_owned()]));
    let VosExpr::Call { callee, args, .. } = &stored.value
    else {
        panic!("expected an insert call")
    };
    assert!(args.is_empty());
    assert!(matches!(callee.as_ref(), VosExpr::Member { name, .. } if name == "insert"));
}

#[test]
fn projects_lists_static_paths_try_and_star_projection() {
    let program = parse_program("let rows = [User { id: 1 }, User { id: 2 }]\nUser::find(rows?).*").expect("Oak projects static paths and projections");
    assert_eq!(program.statements.len(), 1);
    let Some(VosExpr::StarProj { receiver, .. }) = program.result
    else {
        panic!("expected a star projection result")
    };
    let VosExpr::Call { callee, args, .. } = receiver.as_ref()
    else {
        panic!("expected a static call")
    };
    assert!(matches!(args[0], VosExpr::Try { .. }));
    assert!(matches!(callee.as_ref(), VosExpr::Member { sep: VosPathSep::ColonColon, name, .. } if name == "find"));
}

#[test]
fn projects_micro_declarations_with_nested_body_programs() {
    let source = "micro normalize_name(value: utf8) -> utf8 { let name = value.trim() name.lower() }\nlet users = User.filter(x => normalize_name(x.user_name) != \"\").collect()\nusers";
    let program = parse_program(source).expect("Oak projects micro declarations");

    assert_eq!(program.micros.len(), 1);
    let micro = &program.micros[0];
    assert_eq!(micro.name, "normalize_name");
    assert_eq!(micro.parameters.len(), 1);
    assert_eq!(micro.parameters[0].name, "value");
    assert!(matches!(micro.return_type, Some(VosTypeSyntax::Named { ref path, .. }) if path == &["utf8".to_owned()]));
    assert_eq!(micro.body.statements.len(), 1);
    assert!(matches!(micro.body.result, Some(VosExpr::Call { .. })));
    assert_eq!(&source[micro.span.clone()], "micro normalize_name(value: utf8) -> utf8 { let name = value.trim() name.lower() }");
    assert_eq!(&source[micro.body.span.clone()], "{ let name = value.trim() name.lower() }");
    assert_eq!(program.statements.len(), 1);
}

#[test]
fn declaration_bodies_carry_program_projection() {
    let source = "query active_users(status: utf8) { User.filter(x => x.status == status) }";
    let root = parse(source).expect("Oak parses the query declaration");
    let query = &root.declarations[0];
    let program = query.body_program.as_ref().expect("query body is a program");
    assert!(program.statements.is_empty());
    assert!(matches!(program.result, Some(VosExpr::Call { .. })));
    assert_eq!(&source[program.span.clone()], "{ User.filter(x => x.status == status) }");
    assert_eq!(&source[query.body.as_ref().unwrap().span.clone()], "{ User.filter(x => x.status == status) }");
}

#[test]
fn program_projection_rejects_unbalanced_or_incomplete_expressions() {
    for source in ["let value = ", "User.filter(", "x => ", "let = User"] {
        assert!(parse_program(source).is_err(), "accepted incomplete program: {source}");
    }
}

#[test]
fn service_projects_unary_method() {
    let source = "service IdentityService {\n  create(request: CreateRequest) -> Identity\n}";
    let root = parse(source).expect("Oak projects the service body");
    let service = &root.declarations[0];
    assert_eq!(service.kind, VosDeclarationKind::Service);
    assert_eq!(service.name.as_deref(), Some("IdentityService"));
    assert_eq!(service.methods.len(), 1);

    let method = &service.methods[0];
    assert_eq!(method.name, "create");
    assert_eq!(&source[method.name_span.clone()], "create");
    assert_eq!(method.parameters.len(), 1);
    assert_eq!(method.parameters[0].name, "request");
    assert_eq!(method.parameters[0].type_syntax.text, "CreateRequest");
    assert_eq!(method.signature.as_ref().unwrap().text, "(request: CreateRequest)");

    let return_type = method.return_type.as_ref().expect("method return type");
    assert!(!return_type.stream);
    assert_eq!(return_type.type_syntax.text, "-> Identity");
    assert!(matches!(&return_type.type_expr, VosTypeSyntax::Named { path, .. } if path == &["Identity".to_owned()]));
    assert_eq!(&source[return_type.span.clone()], "-> Identity");
}

#[test]
fn service_projects_multiple_methods_and_server_streams() {
    let source = "service FeedService {\n  list(limit: i64) -> [Feed]\n  subscribe(topic: utf8) -> stream<Feed>\n}";
    let root = parse(source).expect("Oak projects streaming methods");
    let methods = &root.declarations[0].methods;
    assert_eq!(methods.iter().map(|method| method.name.as_str()).collect::<Vec<_>>(), ["list", "subscribe"]);

    let list = methods[0].return_type.as_ref().unwrap();
    assert!(!list.stream);
    assert!(matches!(&list.type_expr, VosTypeSyntax::List { element, .. } if matches!(element.as_ref(), VosTypeSyntax::Named { path, .. } if path == &["Feed".to_owned()])));

    let subscribe = methods[1].return_type.as_ref().unwrap();
    assert!(subscribe.stream);
    assert_eq!(subscribe.type_syntax.text, "-> stream<Feed>");
    assert!(matches!(&subscribe.type_expr, VosTypeSyntax::Named { path, .. } if path == &["Feed".to_owned()]));
    assert!(methods[1].span.end <= source.len());
}

#[test]
fn service_preserves_open_attribute_sets_on_service_and_methods() {
    let source = "@route(\"/identity\")\nservice IdentityService {\n  @authorize(\"admin\")\n  [route(\"create\")]\n  create(request: CreateRequest) -> Identity\n}";
    let root = parse(source).expect("Oak preserves service attributes");
    let service = &root.declarations[0];
    assert_eq!(service.attributes.len(), 1);
    assert_eq!(service.attributes[0].name.as_deref(), Some("route"));
    assert_eq!(service.attributes[0].text, "@route(\"/identity\")");
    assert_eq!(&source[service.attributes[0].span.clone()], "@route(\"/identity\")");

    let method = &service.methods[0];
    assert_eq!(method.attributes.len(), 2);
    assert_eq!(method.attributes[0].name.as_deref(), Some("authorize"));
    assert_eq!(method.attributes[0].text, "@authorize(\"admin\")");
    assert_eq!(method.attributes[1].name.as_deref(), Some("route"));
    assert_eq!(method.attributes[1].text, "[route(\"create\")]");
    for attribute in &method.attributes {
        assert_eq!(&source[attribute.span.clone()], attribute.text);
    }
}

#[test]
fn service_attributes_accept_unknown_names_and_arguments() {
    let source = "service Audit {\n  @custom_trace(source = \"edge\", retries = 3)\n  ping(request: Ping) -> Pong\n}";
    let root = parse(source).expect("unknown attributes stay in the open set");
    let method = &root.declarations[0].methods[0];
    assert_eq!(method.attributes[0].name.as_deref(), Some("custom_trace"));
    assert_eq!(method.attributes[0].text, "@custom_trace(source = \"edge\", retries = 3)");
}

#[test]
fn empty_service_projects_an_empty_method_list() {
    let source = "service Empty {\n  # no methods yet\n}";
    let root = parse(source).expect("Oak accepts an empty service");
    let service = &root.declarations[0];
    assert!(service.methods.is_empty());
    assert_eq!(service.body.as_ref().unwrap().text, "{\n  # no methods yet\n}");
}

#[test]
fn service_methods_project_nested_generic_parameter_types() {
    let source = "service Search {\n  find(filter: list<map<utf8, vector<3>>>) -> stream<[User?]>\n}";
    let root = parse(source).expect("Oak projects nested generic types");
    let method = &root.declarations[0].methods[0];
    let parameter = &method.parameters[0];
    assert_eq!(parameter.type_syntax.text, "list<map<utf8, vector<3>>>");
    let VosTypeSyntax::Generic { path, arguments, .. } = &parameter.type_expr
    else {
        panic!("expected a generic parameter type")
    };
    assert_eq!(path, &["list".to_owned()]);
    let VosTypeArgument::Type(VosTypeSyntax::Generic { path, arguments: inner, .. }) = &arguments[0]
    else {
        panic!("expected nested generic arguments")
    };
    assert_eq!(path, &["map".to_owned()]);
    assert_eq!(inner.len(), 2);
    assert!(matches!(&inner[1], VosTypeArgument::Type(VosTypeSyntax::Generic { path, arguments, .. }) if path == &["vector".to_owned()] && matches!(&arguments[0], VosTypeArgument::Literal(value) if value.text == "3")));

    let return_type = method.return_type.as_ref().unwrap();
    assert!(return_type.stream);
    assert_eq!(return_type.type_syntax.text, "-> stream<[User?]>");
    assert!(matches!(&return_type.type_expr, VosTypeSyntax::List { element, .. } if matches!(element.as_ref(), VosTypeSyntax::Optional { inner, .. } if matches!(inner.as_ref(), VosTypeSyntax::Named { path, .. } if path == &["User".to_owned()]))));
}

#[test]
fn service_body_syntax_errors_produce_diagnostics_without_panicking() {
    for source in [
        "service Broken {",
        "service Broken { create(request: Request) Identity }",
        "service Broken { create(request: Request) -> }",
        "service Broken { create(request Request) -> Response }",
        "service Broken { create(request: Request) -> stream<Request }",
        "service Broken { create(request: Request) -> stream<> }",
        "service Broken { 42(request: Request) -> Response }",
    ] {
        assert!(parse(source).is_err(), "accepted malformed service: {source}");
    }
}

#[test]
fn service_surface_round_trips_through_lossless_cst() {
    fn append_text(node: &oak_vos::VosSyntaxNode, text: &mut String) {
        for child in &node.children {
            match child {
                VosSyntaxElement::Node(inner) => append_text(inner, text),
                VosSyntaxElement::Token(token) => text.push_str(&token.text),
            }
        }
    }
    let source =
        "# service surface\n@route(\"/identity\")\nservice IdentityService {\n  # method\n  @authorize(\"admin\")\n  create(request: CreateRequest) -> Identity\n\n  subscribe(topic: utf8) -> stream<Feed>; # trailing\n}\n\ntable User { id: uuid, }\n";
    let root = parse(source).expect("Oak preserves the service surface");
    assert_eq!(root.declarations.len(), 2);
    assert_eq!(root.declarations[0].methods.len(), 2);
    assert_eq!(root.declarations[0].methods[1].return_type.as_ref().unwrap().type_syntax.text, "-> stream<Feed>");
    let mut restored = String::new();
    append_text(&root.syntax, &mut restored);
    assert_eq!(restored, source);
}

#[test]
fn type_depth_has_process_bound() {
    use std::{
        process::Command,
        thread,
        time::{Duration, Instant},
    };
    const CHILD_ENV: &str = "OAK_VOS_TYPE_DEPTH_CHILD";
    if std::env::var_os(CHILD_ENV).is_some() {
        let source = format!("class T {{ field: {}utf8{} }}", "[".repeat(127), "]".repeat(127));
        let root = parse(&source).expect("types below the depth limit are valid");
        assert_eq!(root.declarations[0].fields.len(), 1);
        for wrapper in ["[", "list<"] {
            let closing = if wrapper == "[" { "]" } else { ">" };
            let source = format!("class T {{ field: {}utf8{} }}", wrapper.repeat(4096), closing.repeat(4096));
            assert!(parse(&source).is_err(), "excessive type nesting must be rejected");
        }
        return;
    }
    let mut child = Command::new(std::env::current_exe().unwrap()).args(["--exact", "type_depth_has_process_bound", "--nocapture"]).env(CHILD_ENV, "1").spawn().expect("spawn bounded parser test");
    let deadline = Instant::now() + Duration::from_secs(10);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                assert!(status.success(), "bounded parser child failed: {status}");
                break;
            }
            Ok(None) if Instant::now() < deadline => thread::sleep(Duration::from_millis(10)),
            result => {
                child.kill().expect("terminate parser child");
                child.wait().expect("reap parser child");
                panic!("bounded parser child timed out or failed to poll: {result:?}");
            }
        }
    }
}
