//! Parenthesized arrow parameter lists must parse and build real
//! `FunctionParam`s, including a lone rest parameter `(...args) =>` and the
//! mixed `(a, ...rest) =>` form.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, FunctionParam, Statement, TypeScriptRoot},
};

fn build(source: &str) -> TypeScriptRoot {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let output = TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    output.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn arrow_params(source: &str) -> Vec<FunctionParam> {
    let root = build(source);
    let Some(Statement::VariableDeclaration(decl)) = root.statements.first() else {
        panic!("expected variable declaration: {:#?}", root.statements)
    };
    let value = decl.declarators.first().and_then(|d| d.value.clone()).expect("declarator value");
    let ExpressionKind::ArrowFunction { params, .. } = *value.kind else {
        panic!("expected arrow function, got {:#?}", value.kind)
    };
    params
}

#[test]
fn lone_rest_parameter_arrow_parses_and_builds_param() {
    let params = arrow_params("const a = (...args) => args.length;");
    assert_eq!(params.len(), 1);
    assert!(params[0].is_rest);
    assert_eq!(params[0].name, "args");
}

#[test]
fn mixed_rest_after_named_parameter_arrow() {
    let params = arrow_params("const b = (a, ...rest) => rest.length + a;");
    assert_eq!(params.len(), 2);
    assert!(!params[0].is_rest);
    assert_eq!(params[0].name, "a");
    assert!(params[1].is_rest);
    assert_eq!(params[1].name, "rest");
}

#[test]
fn plain_parenthesized_arrow_still_builds_params() {
    let params = arrow_params("const c = (x) => x;");
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "x");
    assert!(!params[0].is_rest);
}

#[test]
fn parenthesized_expression_is_not_arrow_params() {
    build("const d = (a + b) * c;");
    build("const e = ((x) => x)(1);");
    build("const f = (a, b);");
}
