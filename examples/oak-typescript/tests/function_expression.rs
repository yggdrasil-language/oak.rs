//! Function expressions in initializer / assignment position.
//!
//! The parser already produced `FunctionExpression` nodes, but the builder had
//! no arm for them and fell back to "return the first child expression", which
//! dropped the entire parameterized body. Dependency analysis then saw no
//! references inside `X.create = function (api) { … }`.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn parsed_root(source: &str) -> TypeScriptRoot {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    output.result.expect("source should parse")
}

fn function_expression(kind: ExpressionKind) -> (Option<String>, usize, Vec<Statement>) {
    match kind {
        ExpressionKind::FunctionExpression { name, params, body, .. } => (name, params.len(), body),
        other => panic!("expected function expression, got {other:?}"),
    }
}

#[test]
fn parameterized_function_expression_keeps_body() {
    let root = parsed_root("const x = function (api) { api.el('main'); };\n");
    let Statement::VariableDeclaration(declaration) = root.statements.first().cloned().unwrap() else {
        panic!("expected variable declaration");
    };
    let value = declaration.value().expect("initializer");
    let (name, params, body) = function_expression(*value.kind.clone());
    assert!(name.is_none(), "anonymous function expression should have no name");
    assert_eq!(params, 1);
    assert_eq!(body.len(), 1, "{body:#?}");
    let Statement::ExpressionStatement(statement) = &body[0] else {
        panic!("expected expression statement, got {:?}", body[0]);
    };
    let ExpressionKind::CallExpression { .. } = &*statement.expression.kind else {
        panic!("expected call expression, got {:?}", statement.expression.kind);
    };
}

#[test]
fn named_function_expression_assigned_to_member_keeps_body_and_name() {
    let root = parsed_root("P.create = function named(api) { api.component(Greeting); };\n");
    let Some(Statement::ExpressionStatement(statement)) = root.statements.first().cloned() else {
        panic!("expected expression statement");
    };
    let ExpressionKind::AssignmentExpression { right, .. } = &*statement.expression.kind else {
        panic!("expected assignment, got {:?}", statement.expression.kind);
    };
    let value = right.as_ref();
    let (name, params, body) = function_expression(*value.kind.clone());
    assert_eq!(name.as_deref(), Some("named"));
    assert_eq!(params, 1);
    assert_eq!(body.len(), 1, "{body:#?}");
}
