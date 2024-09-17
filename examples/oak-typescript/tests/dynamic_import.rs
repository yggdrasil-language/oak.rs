use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Expression, ExpressionKind, Statement},
};

fn parse(source: &str) -> oak_typescript::ast::TypeScriptRoot {
    let lang = TypeScriptLanguage { jsx: false, ..TypeScriptLanguage::standard() };
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.expect("root")
}

fn specifier(expression: &Expression) -> &str {
    let ExpressionKind::ImportExpression { module_specifier } = &*expression.kind else {
        panic!("expected an import expression, got {:?}", expression.kind);
    };
    let ExpressionKind::StringLiteral(text) = &*module_specifier.kind else {
        panic!("expected a literal specifier, got {:?}", module_specifier.kind);
    };
    text
}

#[test]
fn import_expression_statement_carries_the_specifier() {
    let root = parse("import('./lazy.js');");
    let Some(Statement::ExpressionStatement(statement)) = root.statements.first() else {
        panic!("expected an expression statement, got {:#?}", root.statements);
    };
    assert_eq!(specifier(&statement.expression), "./lazy.js");
}

#[test]
fn arrow_const_keeps_import_expression_inside_the_body() {
    let root = parse("const load = () => import('./lazy.js');");
    let Some(Statement::VariableDeclaration(declaration)) = root.statements.first() else {
        panic!("expected a variable declaration, got {:#?}", root.statements);
    };
    let init = declaration.declarators[0].value.as_ref().expect("initializer");
    let ExpressionKind::ArrowFunction { body, .. } = &*init.kind else {
        panic!("expected an arrow function, got {:?}", init.kind);
    };
    let body = body.as_ref();
    let expression = match body {
        Statement::ExpressionStatement(statement) => &statement.expression,
        Statement::ReturnStatement(ret) => ret.argument.as_ref().expect("return argument"),
        other => panic!("unexpected arrow body {other:?}"),
    };
    assert_eq!(specifier(expression), "./lazy.js");
}

/// Control: calling an empty arrow function with a string argument is a plain
/// call. It must never be folded into a dynamic import edge.
#[test]
fn empty_arrow_call_is_not_an_import_expression() {
    let root = parse("(() => {})('./lazy.js');");
    let Some(Statement::ExpressionStatement(statement)) = root.statements.first() else {
        panic!("expected an expression statement, got {:#?}", root.statements);
    };
    let ExpressionKind::CallExpression { func, args } = &*statement.expression.kind else {
        panic!("expected a call, got {:?}", statement.expression.kind);
    };
    assert!(matches!(&*func.kind, ExpressionKind::ArrowFunction { .. }), "{:?}", func.kind);
    assert_eq!(args.len(), 1);
}

#[test]
fn import_meta_stays_a_meta_property() {
    let root = parse("const base = import.meta.url;");
    let Some(Statement::VariableDeclaration(declaration)) = root.statements.first() else {
        panic!("expected a variable declaration");
    };
    let init = declaration.declarators[0].value.as_ref().expect("initializer");
    let ExpressionKind::MemberExpression { object, property, .. } = &*init.kind else {
        panic!("expected a member expression, got {:?}", init.kind);
    };
    assert!(matches!(&*object.kind, ExpressionKind::MetaProperty { meta, property } if meta == "import" && property == "meta"));
    assert!(matches!(&*property.kind, ExpressionKind::Identifier(name) if name == "url"));
}
