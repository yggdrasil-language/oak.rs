use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage, ast::{ExpressionKind, Statement}};

fn call_shape(source: &str) -> (String, String, Vec<String>) {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    let root = result.result.expect("root");
    let Some(Statement::ExpressionStatement(stmt)) = root.statements.first() else {
        panic!("{source}: expected expression statement, got {:?}", root.statements);
    };
    let ExpressionKind::CallExpression { func, args } = &*stmt.expression.kind else {
        panic!("{source}: expected call, got {:?}", stmt.expression.kind);
    };
    let callee = match &*func.kind {
        ExpressionKind::MemberExpression { object, property, .. } => {
            let obj = match &*object.kind {
                ExpressionKind::Identifier(n) => n.clone(),
                other => panic!("{source}: expected identifier object, got {other:?}"),
            };
            let prop = match &*property.kind {
                ExpressionKind::Identifier(n) => n.clone(),
                other => panic!("{source}: expected identifier property, got {other:?}"),
            };
            format!("{obj}.{prop}")
        }
        ExpressionKind::Identifier(n) => n.clone(),
        other => panic!("{source}: unexpected callee {other:?}"),
    };
    let arg_names = args.iter().map(|a| format!("{:?}", a.kind)).collect();
    (callee, String::new(), arg_names)
}

#[test]
fn member_calls_with_keyword_property_names_keep_callee_and_args() {
    // `get`, `from`, `read`, `type` are keyword token kinds in the lexer; the
    // builder must still surface them as member-call properties.
    for source in [
        "map.get(name);",
        "Buffer.from(c);",
        "Array.from(line);",
        "scrollPositions.get(navKey);",
        "obj.set(k, v);",
        "reader.read();",
        "str.startsWith(\"event:\");",
    ] {
        let (callee, _, args) = call_shape(source);
        let expected = source.trim_end_matches(';').rsplit_once('(').unwrap().0.to_string();
        assert_eq!(callee, expected, "{source}");
        if source.contains("set(k, v)") {
            assert_eq!(args.len(), 2, "{source}");
        }
        else if source.ends_with("();") {
            assert!(args.is_empty(), "{source}");
        }
        else {
            assert_eq!(args.len(), 1, "{source}");
        }
    }
}

#[test]
fn contextual_keyword_identifiers_parse_in_expression_position() {
    // `require`, `type`, `async` are keyword token kinds but valid identifiers
    // in expression position. `type`/`async` at statement start are reserved
    // for type aliases / async functions, so exercise them as arguments.
    for (source, callee) in [
        ("require('./m');", "require"),
        ("f(type);", "f"),
        ("f(async);", "f"),
        ("f(from);", "f"),
    ] {
        let (got, _, args) = call_shape(source);
        assert_eq!(got, callee, "{source}");
        assert_eq!(args.len(), 1, "{source}");
    }
}

#[test]
fn member_call_in_variable_declaration_keeps_callee_and_args() {
    let lang = TypeScriptLanguage::standard();
    let source = "const v = map.get(name);";
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{:#?}", result.diagnostics);
    let root = result.result.expect("root");
    let Some(Statement::VariableDeclaration(decl)) = root.statements.first() else {
        panic!("expected var decl, got {:?}", root.statements);
    };
    let init = decl.declarators[0].value.clone().expect("init");
    let ExpressionKind::CallExpression { func, args } = *init.kind else {
        panic!("expected call, got {:?}", init.kind);
    };
    let ExpressionKind::MemberExpression { object, property, .. } = *func.kind else {
        panic!("expected member callee, got {:?}", func.kind);
    };
    assert!(matches!(&*object.kind, ExpressionKind::Identifier(n) if n == "map"));
    assert!(matches!(&*property.kind, ExpressionKind::Identifier(n) if n == "get"));
    assert_eq!(args.len(), 1, "one argument");
    assert!(matches!(&*args[0].kind, ExpressionKind::Identifier(n) if n == "name"));
}
