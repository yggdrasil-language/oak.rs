use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, ModifierKind, Statement, TypeScriptRoot},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn root_statements(root: &TypeScriptRoot) -> &[Statement] {
    &root.statements
}

/// `new URL('<literal>', import.meta.url)` must become a real NewExpression
/// whose second argument is `MemberExpression { object: MetaProperty, .. }`.
#[test]
fn new_url_with_import_meta_url() {
    let source = "const u = new URL('./asset.wasm', import.meta.url);";
    let root = build(source);
    let Statement::VariableDeclaration(decl) = &root_statements(&root)[0]
    else {
        panic!("expected variable declaration: {:?}", root_statements(&root))
    };
    let value = decl.value.as_ref().expect("initializer must exist");
    let ExpressionKind::NewExpression { func, args } = &*value.kind
    else {
        panic!("expected new expression: {:?}", value.kind)
    };
    assert!(matches!(&*func.kind, ExpressionKind::Identifier(name) if name == "URL"));
    assert_eq!(args.len(), 2, "import.meta.url argument must survive the build");
    let ExpressionKind::MemberExpression { object, property, .. } = &*args[1].kind
    else {
        panic!("expected member expression: {:?}", args[1].kind)
    };
    let ExpressionKind::MetaProperty { meta, property: meta_property } = &*object.kind
    else {
        panic!("expected meta property: {:?}", object.kind)
    };
    assert_eq!(meta, "import");
    assert_eq!(meta_property, "meta");
    assert!(matches!(&*property.kind, ExpressionKind::Identifier(name) if name == "url"));
}

/// Standalone `import.meta` and `import.meta.url` in expression positions.
#[test]
fn import_meta_member_access_positions() {
    let root = build("const m = import.meta;\nimport.meta.url;\nconsole.log(import.meta.url);");
    let stmts = root_statements(&root);
    assert_eq!(stmts.len(), 3, "{stmts:#?}");

    let Statement::VariableDeclaration(decl) = &stmts[0]
    else {
        panic!("stmt 0: {:?}", stmts[0])
    };
    let value = decl.value.as_ref().expect("import.meta initializer");
    assert!(matches!(&*value.kind, ExpressionKind::MetaProperty { meta, property } if meta == "import" && property == "meta"));

    let Statement::ExpressionStatement(stmt) = &stmts[1]
    else {
        panic!("stmt 1: {:?}", stmts[1])
    };
    let ExpressionKind::MemberExpression { object, .. } = &*stmt.expression.kind
    else {
        panic!("stmt 1 expression: {:?}", stmt.expression.kind)
    };
    assert!(matches!(&*object.kind, ExpressionKind::MetaProperty { .. }));

    let Statement::ExpressionStatement(stmt) = &stmts[2]
    else {
        panic!("stmt 2: {:?}", stmts[2])
    };
    let ExpressionKind::CallExpression { args, .. } = &*stmt.expression.kind
    else {
        panic!("stmt 2 expression: {:?}", stmt.expression.kind)
    };
    assert_eq!(args.len(), 1);
    let ExpressionKind::MemberExpression { object, .. } = &*args[0].kind
    else {
        panic!("stmt 2 argument: {:?}", args[0].kind)
    };
    assert!(matches!(&*object.kind, ExpressionKind::MetaProperty { .. }));
}

/// `import.meta` must not be confused with the dynamic `import(...)` call.
#[test]
fn dynamic_import_still_parses() {
    let root = build("import('./lazy.js').then((m) => m);");
    let stmts = root_statements(&root);
    // The statement-head dynamic import keeps its existing fingerprint shape;
    // this test only pins that the meta-property change did not break it.
    assert!(!stmts.is_empty());
}

fn await_spans_text(root: &TypeScriptRoot, source: &str) -> Vec<String> {
    root.top_level_await_spans.iter().map(|span| source[span.start..span.end].to_string()).collect()
}

#[test]
fn top_level_await_is_recorded() {
    let positives: &[&str] = &["const x = await f();", "await f();", "if (await f()) { g(); }", "try { await f(); } catch (e) { }", "for await (const x of xs) { }", "for await (x of xs) { }", "let a = 1;\nawait f();\nawait g();"];
    for source in positives {
        let root = build(source);
        assert!(root.has_top_level_await(), "{source}: expected top-level await");
        assert!(!root.top_level_await_spans.is_empty(), "{source}: spans must be recorded");
        for text in await_spans_text(&root, source) {
            assert!(text.trim_start().starts_with("await"), "{source}: span text {text:?} must start at the await keyword");
        }
    }

    let root = build("const x = await f();");
    assert_eq!(await_spans_text(&root, "const x = await f();"), vec!["await f()".to_string()]);

    let root = build("for await (const x of xs) { }");
    assert_eq!(await_spans_text(&root, "for await (const x of xs) { }"), vec!["await".to_string()]);
}

#[test]
fn await_inside_functions_is_not_top_level() {
    let negatives: &[&str] = &[
        "async function main() { await f(); }",
        "const g = async () => { await f(); };",
        "function h() { return await f(); }",
        "class C { async m() { await f(); } }",
        "const p = xs.map(async (x) => await f(x));",
        "async function outer() { for await (const x of xs) { } }",
    ];
    for source in negatives {
        let root = build(source);
        assert!(!root.has_top_level_await(), "{source}: must not report top-level await");
        assert!(root.top_level_await_spans.is_empty(), "{source}: spans must be empty");
    }
}

#[test]
fn for_of_without_await_is_not_tla() {
    let root = build("for (const x of xs) { }");
    assert!(!root.has_top_level_await());
    let Statement::ForOfStatement(stmt) = &root_statements(&root)[0]
    else {
        panic!("expected for-of statement: {:?}", root_statements(&root))
    };
    assert!(stmt.await_marker.is_none());
    assert!(!stmt.is_await());
}

#[test]
fn for_await_records_marker_on_ast_statement() {
    let root = build("for await (const x of xs) { }");
    let Statement::ForOfStatement(stmt) = &root_statements(&root)[0]
    else {
        panic!("expected for-of statement: {:?}", root_statements(&root))
    };
    let marker = stmt.await_marker.as_ref().expect("await marker");
    assert_eq!(marker.kind, ModifierKind::Await);
    assert!(stmt.is_await());
}

#[test]
fn await_expression_is_a_real_node() {
    let root = build("const x = await f();");
    let Statement::VariableDeclaration(decl) = &root_statements(&root)[0]
    else {
        panic!("decl")
    };
    let value = decl.value.as_ref().expect("value");
    let ExpressionKind::AwaitExpression(argument) = &*value.kind
    else {
        panic!("expected await expression: {:?}", value.kind)
    };
    let ExpressionKind::CallExpression { func, .. } = &*argument.kind
    else {
        panic!("argument")
    };
    assert!(matches!(&*func.kind, ExpressionKind::Identifier(name) if name == "f"));
}
