use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, Statement},
};

fn build(source: &str) -> oak_typescript::ast::TypeScriptRoot {
    let lang = TypeScriptLanguage { jsx: false, ..TypeScriptLanguage::standard() };
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.expect("root")
}

fn single_init(source: &str) -> oak_typescript::ast::Expression {
    let root = build(source);
    let Some(Statement::VariableDeclaration(decl)) = root.statements.first() else {
        panic!("{source}: expected variable declaration, got {:?}", root.statements);
    };
    decl.declarators[0].value.clone().expect("init")
}

#[test]
fn satisfies_parses_as_dedicated_expression() {
    for source in ["const cfg = value satisfies Config;", "const cfg = value satisfies Config | null;"] {
        let init = single_init(source);
        match &*init.kind {
            ExpressionKind::SatisfiesExpression { expression, .. } => {
                assert!(matches!(&*expression.kind, ExpressionKind::Identifier(n) if n == "value"), "{source}");
            }
            other => panic!("{source}: expected satisfies, got {other:?}"),
        }
    }
}

#[test]
fn satisfies_parses_in_call_argument() {
    let root = build("register(name satisfies string);");
    let Some(Statement::ExpressionStatement(stmt)) = root.statements.first() else {
        panic!("expected expression statement");
    };
    let ExpressionKind::CallExpression { args, .. } = &*stmt.expression.kind else {
        panic!("expected call, got {:?}", stmt.expression.kind);
    };
    assert_eq!(args.len(), 1);
    assert!(matches!(&*args[0].kind, ExpressionKind::SatisfiesExpression { .. }), "{:?}", args[0].kind);
}

#[test]
fn satisfies_keeps_plain_identifier_usage() {
    let root = build("const satisfies = 1;\nf(satisfies);");
    assert_eq!(root.statements.len(), 2, "{:?}", root.statements);
    let Some(Statement::ExpressionStatement(stmt)) = root.statements.get(1) else {
        panic!("expected expression statement, got {:?}", root.statements);
    };
    let ExpressionKind::CallExpression { args, .. } = &*stmt.expression.kind else {
        panic!("expected call, got {:?}", stmt.expression.kind);
    };
    assert!(matches!(&*args[0].kind, ExpressionKind::Identifier(n) if n == "satisfies"));
}

#[test]
fn prefix_assertion_parses_as_type_assertion() {
    for (source, want) in [("const el = <HTMLElement>document.body;", "document"), ("const label = <string>cfg.name;", "cfg")] {
        let init = single_init(source);
        match &*init.kind {
            ExpressionKind::TypeAssertionExpression { expression, .. } => {
                assert!(matches!(&*expression.kind, ExpressionKind::MemberExpression { .. }), "{source}: {expression:?}");
            }
            other => panic!("{source}: expected type assertion, got {other:?} (want member on {want})"),
        }
    }
}

#[test]
fn prefix_assertion_parses_in_call_argument_and_nested_generics() {
    let root = build("resolve(<FractalWasm>mod.default);\nconst m = <Map<string, number>>cache.map;");
    assert_eq!(root.statements.len(), 2, "{:?}", root.statements);
    let Some(Statement::ExpressionStatement(stmt)) = root.statements.first() else {
        panic!("expected expression statement");
    };
    let ExpressionKind::CallExpression { args, .. } = &*stmt.expression.kind else {
        panic!("expected call, got {:?}", stmt.expression.kind);
    };
    assert!(matches!(&*args[0].kind, ExpressionKind::TypeAssertionExpression { .. }), "{:?}", args[0].kind);
    let init = match root.statements.get(1) {
        Some(Statement::VariableDeclaration(decl)) => decl.declarators[0].value.clone().expect("init"),
        other => panic!("expected variable declaration, got {other:?}"),
    };
    assert!(matches!(&*init.kind, ExpressionKind::TypeAssertionExpression { .. }), "{:?}", init.kind);
}

#[test]
fn prefix_assertion_does_not_split_module_statements() {
    // The pack layer slices top-level statements by span; an assertion must not
    // break statement boundaries (regression: `const forced = <` + junk).
    let source = "export const forced = <FractalWasm>resolved;\nconst plain = resolved;\n";
    let root = build(source);
    assert_eq!(root.statements.len(), 2, "{:?}", root.statements);
    let Some(Statement::ExportDeclaration(export)) = root.statements.first() else {
        panic!("expected export declaration, got {:?}", root.statements);
    };
    let text = &source[export.span.start..export.span.end];
    assert!(text.contains("<FractalWasm>resolved"), "{text}");
}

#[test]
fn generic_arrow_and_comparisons_stay_intact() {
    // `<T>(x: T) => x` stays a generic arrow, not an assertion.
    let init = single_init("const g = <T>(x: T) => x;");
    match &*init.kind {
        ExpressionKind::ArrowFunction { type_params, .. } => assert_eq!(type_params.len(), 1),
        other => panic!("expected arrow, got {other:?}"),
    }
    // `a < b` stays a comparison.
    let init = single_init("const c = a < b;");
    assert!(matches!(&*init.kind, ExpressionKind::BinaryExpression { .. }), "{:?}", init.kind);
    // Generic call type arguments: the expression parser keeps them inside one
    // statement with no diagnostics (VMZ erases the type arguments at the token
    // layer); the statement must not split or error.
    let source = "identity<string>('x');";
    let root = build(source);
    assert_eq!(root.statements.len(), 1, "{:?}", root.statements);
    let span = root.statements[0].span();
    assert_eq!(&source[span.start..span.end], source.trim_end(), "statement must cover the whole input");
}
