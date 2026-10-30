//! Function parameters expose the complete binding facts: rest markers,
//! destructuring patterns with their bound names, default values, optional
//! markers and type annotations.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, FunctionParam, Statement, TypeScriptRoot},
    formatter::{FormatOptions, format_source},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn params_of(source: &str) -> Vec<FunctionParam> {
    let root = build(source);
    let Some(Statement::FunctionDeclaration(declaration)) = root.statements.first() else {
        panic!("expected a function declaration: {:#?}", root.statements)
    };
    declaration.params.clone()
}

fn print(source: &str) -> String {
    format_source(source, &FormatOptions::default()).expect("format")
}

#[test]
fn plain_parameters_keep_names() {
    let params = params_of("function f(a, b) {}");
    assert_eq!(params.len(), 2);
    assert_eq!(params[0].name, "a");
    assert_eq!(params[0].binding_names, vec!["a".to_string()]);
    assert_eq!(params[1].name, "b");
    assert!(!params[0].is_rest);
    assert!(params[0].pattern_text.is_none());
    assert!(params[0].default.is_none());
}

#[test]
fn rest_parameter_marks_is_rest_and_keeps_name() {
    let params = params_of("function f(...args) {}");
    assert_eq!(params.len(), 1, "the rest run must stay one parameter: {params:#?}");
    assert!(params[0].is_rest);
    assert_eq!(params[0].name, "args");
    assert_eq!(params[0].binding_names, vec!["args".to_string()]);
    assert_eq!(print("function f(...args) {}"), "function f(...args) {}");
}

#[test]
fn destructured_parameter_resolves_bound_names() {
    let params = params_of("function f({ a, b: c, ...rest }) {}");
    assert_eq!(params.len(), 1, "the pattern must stay one parameter: {params:#?}");
    let param = &params[0];
    assert_eq!(param.name, "", "a pattern has no simple name");
    assert_eq!(param.pattern_text.as_deref(), Some("{ a, b: c, ...rest }"));
    assert_eq!(param.binding_names, vec!["a".to_string(), "c".to_string(), "rest".to_string()]);
}

#[test]
fn default_values_are_expressions() {
    let params = params_of("function f(x = 1, y = a + b) {}");
    assert_eq!(params.len(), 2);
    let default_x = params[0].default.as_ref().expect("x default");
    assert!(matches!(default_x.kind.as_ref(), ExpressionKind::NumericLiteral(v) if *v == 1.0));
    let default_y = params[1].default.as_ref().expect("y default");
    assert!(matches!(default_y.kind.as_ref(), ExpressionKind::BinaryExpression { .. }));
    assert_eq!(params[0].binding_names, vec!["x".to_string()]);
}

#[test]
fn optional_and_typed_parameter_round_trip() {
    let params = params_of("function f(a?: string, b: number = 2) {}");
    assert!(params[0].is_optional());
    assert!(params[0].ty.is_some());
    assert!(!params[1].is_optional());
    assert!(params[1].ty.is_some());
    assert!(params[1].default.is_some());
    assert_eq!(print("function f(a?: string, b: number = 2) {}"), "function f(a?: string, b: number = 2) {}");
}

#[test]
fn arrow_destructured_and_rest_parameters_round_trip() {
    assert_eq!(print("const f = ({ a, b }) => a + b;"), "const f = ({ a, b }) => a + b;");
    assert_eq!(print("const g = (...xs) => xs;"), "const g = (...xs) => xs;");
    assert_eq!(print("const h = (x = 1) => x;"), "const h = (x = 1) => x;");
}
