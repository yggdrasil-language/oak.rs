//! Named functions may use contextual keywords (`get`, `set`, `type`, …) as
//! their name. The lexer emits dedicated tokens for those words, so the
//! parser wraps the token in an `IdentifierName` node and the builder's
//! whole-node text fallback recovers the name.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn function_name(source: &str) -> String {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    match root.statements.first().cloned() {
        Some(Statement::FunctionDeclaration(decl)) => decl.name,
        other => panic!("expected function declaration, got {other:?}"),
    }
}

#[test]
fn function_named_set() {
    assert_eq!(function_name("function set (options) { }"), "set");
}

#[test]
fn async_function_named_get() {
    assert_eq!(function_name("async function get(code, options) { }"), "get");
}

#[test]
fn function_named_type() {
    assert_eq!(function_name("function type() {}"), "type");
}

#[test]
fn named_function_expression_with_keyword_name_has_no_diagnostics() {
    let source = "LinkifyIt.prototype.set = function set (options) { return this };";
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
}

#[test]
fn plain_named_function_still_works() {
    assert_eq!(function_name("function foo() {}"), "foo");
}

#[test]
fn anonymous_and_generator_functions_still_parse() {
    for source in ["(function () {})();", "function* gen() {}", "export default function () {}"] {
        let output = parse(source);
        assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    }
}
