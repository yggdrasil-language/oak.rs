//! Variable declarations expose one `VariableDeclarator` per declarator, each
//! carrying the binding identifiers that the CST binding pattern really
//! introduces (aliases, rest elements, array holes, nested patterns and
//! defaults all resolve to their bound names).

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Statement, TypeScriptRoot},
    formatter::{FormatOptions, format_source},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn variable_declaration(source: &str) -> oak_typescript::ast::VariableDeclaration {
    let root = build(source);
    let Some(Statement::VariableDeclaration(declaration)) = root.statements.first() else {
        panic!("expected a variable declaration: {:#?}", root.statements)
    };
    declaration.clone()
}

fn print(source: &str) -> String {
    format_source(source, &FormatOptions::default()).expect("format")
}

#[test]
fn single_declarator_keeps_binding_and_initializer_span() {
    let source = "const a = 1;";
    let declaration = variable_declaration(source);

    assert_eq!(declaration.declarators.len(), 1, "one declarator expected");
    let declarator = &declaration.declarators[0];
    assert_eq!(declarator.binding_names, vec!["a".to_string()]);
    assert_eq!(declarator.pattern_text, None, "a plain binding has no pattern text");
    assert_eq!(source[declarator.span.start..declarator.span.end].trim(), "a = 1");
    assert_eq!(declaration.name(), "a");

    assert_eq!(print(source), source, "printer must keep the single declarator");
}

#[test]
fn multiple_declarators_keep_their_own_bindings_and_initializers() {
    let source = "const a = 1, b = 2;";
    let declaration = variable_declaration(source);

    assert_eq!(declaration.declarators.len(), 2, "both declarators must survive: {declaration:#?}");
    let [first, second] = declaration.declarators.as_slice() else {
        panic!("two declarators expected")
    };
    assert_eq!(first.binding_names, vec!["a".to_string()]);
    assert_eq!(second.binding_names, vec!["b".to_string()]);
    assert_eq!(source[first.span.start..first.span.end].trim(), "a = 1");
    assert_eq!(source[second.span.start..second.span.end].trim(), "b = 2");

    assert_eq!(print(source), source, "printer must print every declarator");
}

#[test]
fn object_pattern_resolves_aliases_rest_and_nested_targets() {
    let source = "const { a, b: c, ...rest } = require('./m');";
    let declaration = variable_declaration(source);

    assert_eq!(declaration.declarators.len(), 1);
    let declarator = &declaration.declarators[0];
    assert_eq!(declarator.binding_names, vec!["a".to_string(), "c".to_string(), "rest".to_string()]);
    assert_eq!(declarator.pattern_text.as_deref(), Some("{ a, b: c, ...rest }"));
    assert_eq!(source[declarator.span.start..declarator.span.end].trim(), "{ a, b: c, ...rest } = require('./m')");

    assert_eq!(print(source), source, "the destructuring pattern must be printed as written");

    let nested = variable_declaration("const { a: { b = 1 }, c: [d] } = m;");
    assert_eq!(nested.declarators[0].binding_names, vec!["b".to_string(), "d".to_string()]);
}

#[test]
fn array_pattern_skips_holes_and_keeps_rest() {
    let source = "const [x, , ...ys] = arr;";
    let declaration = variable_declaration(source);

    assert_eq!(declaration.declarators.len(), 1);
    let declarator = &declaration.declarators[0];
    assert_eq!(declarator.binding_names, vec!["x".to_string(), "ys".to_string()]);
    assert_eq!(declarator.pattern_text.as_deref(), Some("[x, , ...ys]"));
    assert_eq!(source[declarator.span.start..declarator.span.end].trim(), "[x, , ...ys] = arr");

    // The token-gap formatter collapses the space in an elision hole, but keeps
    // the hole and the rest element. `pattern_text` above is untouched.
    assert_eq!(print(source), "const [x,, ...ys] = arr;");
}

#[test]
fn identifier_initializer_is_not_a_second_binding() {
    let declaration = variable_declaration("const x = y;");
    assert_eq!(declaration.declarators.len(), 1);
    assert_eq!(declaration.declarators[0].binding_names, vec!["x".to_string()]);

    let declaration = variable_declaration("let a = b, c = d;");
    assert_eq!(declaration.declarators.len(), 2);
    assert_eq!(declaration.declarators[0].binding_names, vec!["a".to_string()]);
    assert_eq!(declaration.declarators[1].binding_names, vec!["c".to_string()]);
}

#[test]
fn exported_destructuring_declaration_reaches_the_root() {
    let source = "export const { a } = x;";
    let root = build(source);

    let Some(Statement::ExportDeclaration(export)) = root.statements.first() else {
        panic!("expected an export declaration: {:#?}", root.statements)
    };
    let Some(Statement::VariableDeclaration(declaration)) = export.declaration.as_deref() else {
        panic!("exported declaration must be a variable declaration: {export:#?}")
    };
    assert_eq!(declaration.declarators.len(), 1);
    assert_eq!(declaration.declarators[0].binding_names, vec!["a".to_string()]);
    assert_eq!(declaration.name(), "a");

    assert_eq!(print(source), source);
}
