//! Type parameter lists accept a `const` modifier everywhere they are parsed:
//! declarations (`function f<const T>`, `class Box<const T>`, methods,
//! interfaces) already scan the balanced list, but an expression-position
//! generic arrow (`<const T>(x: T) => x`) fell into the JSX element parser,
//! where the keyword attribute name made the attribute loop spin forever
//! allocating empty nodes. JSX mode now recognizes the generic-arrow shape
//! first, and the attribute loop guarantees progress.

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

fn build(source: &str) -> TypeScriptRoot {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    output.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn arrow_initializer(source: &str) -> ExpressionKind {
    let root = build(source);
    let Some(Statement::VariableDeclaration(decl)) = root.statements.first() else {
        panic!("expected variable declaration: {:#?}", root.statements)
    };
    let value = decl.declarators.first().and_then(|d| d.value.clone()).expect("declarator value");
    (*value.kind).clone()
}

#[test]
fn declaration_positions_accept_const_type_parameter() {
    build("function f<const T>(x: T): T {\n  return x;\n}");
    build("class Box<const T> {\n  get(): T {\n    return this.value;\n  }\n}");
    build("interface Box<const T> {\n  value: T;\n}");
    build("class C {\n  m<const T>(x: T): T {\n    return x;\n  }\n}");
}

#[test]
fn generic_arrow_with_const_type_parameter() {
    let kind = arrow_initializer("const f = <const T>(x: T) => x;");
    let ExpressionKind::ArrowFunction { params, .. } = kind else {
        panic!("expected arrow function, got {kind:#?}")
    };
    assert_eq!(params.len(), 1);
    assert_eq!(params[0].name, "x");
}

#[test]
fn generic_arrow_with_const_type_parameter_and_return_annotation() {
    let kind = arrow_initializer("const f = <const T>(x: T): T => x;");
    assert!(matches!(kind, ExpressionKind::ArrowFunction { .. }), "got {kind:?}");
}

#[test]
fn generic_arrow_with_const_constraint() {
    let kind = arrow_initializer("const f = <const T extends string>(x: T) => x.length;");
    assert!(matches!(kind, ExpressionKind::ArrowFunction { .. }), "got {kind:?}");
}

#[test]
fn plain_generic_arrow_still_parses_in_jsx_mode() {
    let kind = arrow_initializer("const f = <T>(x: T): T => x;");
    let ExpressionKind::ArrowFunction { params, .. } = kind else {
        panic!("expected arrow function, got {kind:#?}")
    };
    assert_eq!(params.len(), 1);
}

#[test]
fn jsx_element_is_not_an_arrow() {
    // A real element follows `<` with a name, not a type parameter list that
    // ends in `(...)` + `=>`; the disambiguation must not swallow JSX.
    build("const el = <div class=\"x\">hello</div>;");
}

#[test]
fn keyword_attribute_name_terminates_with_diagnostic() {
    // `<div 123>` has no identifier where the attribute name belongs. The
    // attribute loop must make progress and report instead of spinning.
    let output = parse("const el = <div 123>x</div>;");
    assert!(!output.diagnostics.is_empty(), "expected a diagnostic for the invalid attribute");
}

#[test]
fn generic_arrow_after_expression_terminates() {
    // Regression: `<const T>(x: T) => x` used to allocate unbounded empty
    // JSX attribute nodes until the process died.
    let output = parse("export const pick = <const T>(x: T) => x;");
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
    assert!(output.result.is_ok());
}
