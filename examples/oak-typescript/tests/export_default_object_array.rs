//! `export default` followed by an object literal must parse the literal as
//! an expression, not as a block statement. Reading `{` as a block turned
//! every property `:` into a diagnostic and broke array members after the
//! first comma.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, ObjectProperty, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn default_exported_kind(source: &str) -> ExpressionKind {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    let Some(Statement::ExportDeclaration(export)) = root.statements.first().cloned() else {
        panic!("expected export declaration in {source}");
    };
    assert!(export.is_default(), "expected default modifier in {source}");
    let inner = export.declaration.expect("default payload");
    let Statement::ExpressionStatement(expr) = *inner else {
        panic!("expected expression statement, got {inner:?}");
    };
    *expr.expression.kind
}

fn property_value<'a>(properties: &'a [ObjectProperty], name: &str) -> &'a ExpressionKind {
    for prop in properties {
        if let ObjectProperty::Property { name: key, value, .. } = prop {
            if key == name {
                return &value.kind;
            }
        }
    }
    panic!("missing property `{name}`");
}

#[test]
fn object_with_string_array_property() {
    let kind = default_exported_kind("export default { rules: ['x', 'y'] }");
    let ExpressionKind::ObjectLiteral { properties } = kind else {
        panic!("expected object literal, got {kind:?}");
    };
    assert_eq!(properties.len(), 1);
    let ExpressionKind::ArrayLiteral { elements } = property_value(&properties, "rules") else {
        panic!("expected array literal value");
    };
    assert_eq!(elements.len(), 2);
}

#[test]
fn object_with_identifier_array_property() {
    let kind = default_exported_kind("export default { rules: [x, y] }");
    let ExpressionKind::ObjectLiteral { properties } = kind else {
        panic!("expected object literal, got {kind:?}");
    };
    let ExpressionKind::ArrayLiteral { elements } = property_value(&properties, "rules") else {
        panic!("expected array literal value");
    };
    assert_eq!(elements.len(), 2);
}

#[test]
fn nested_objects_and_arrays() {
    let kind = default_exported_kind("export default { a: { b: [1, 2, 3] }, c: [{ d: 'e' }, { f: 'g' }] }");
    let ExpressionKind::ObjectLiteral { properties } = kind else {
        panic!("expected object literal, got {kind:?}");
    };
    assert_eq!(properties.len(), 2);
    let ExpressionKind::ObjectLiteral { properties: inner } = property_value(&properties, "a") else {
        panic!("expected nested object for `a`");
    };
    let ExpressionKind::ArrayLiteral { elements } = property_value(inner, "b") else {
        panic!("expected array for `a.b`");
    };
    assert_eq!(elements.len(), 3);
    let ExpressionKind::ArrayLiteral { elements: objs } = property_value(&properties, "c") else {
        panic!("expected array for `c`");
    };
    assert_eq!(objs.len(), 2);
    assert!(matches!(*objs[0].kind, ExpressionKind::ObjectLiteral { .. }));
}

#[test]
fn multiline_object_with_trailing_comma() {
    let kind = default_exported_kind("export default {\n  rules: ['a', 'b'],\n  fmt: { x: 1 },\n}\n");
    let ExpressionKind::ObjectLiteral { properties } = kind else {
        panic!("expected object literal, got {kind:?}");
    };
    assert_eq!(properties.len(), 2);
}

#[test]
fn default_export_array_literal_still_parses() {
    let kind = default_exported_kind("export default [1, 2, 3]");
    let ExpressionKind::ArrayLiteral { elements } = kind else {
        panic!("expected array literal, got {kind:?}");
    };
    assert_eq!(elements.len(), 3);
}

#[test]
fn default_function_and_class_still_parse_as_declarations() {
    let output = parse("export default function foo() {}\nexport default class Bar {}\n");
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    assert!(matches!(root.statements[0], Statement::ExportDeclaration(_)));
    assert!(matches!(root.statements[1], Statement::ExportDeclaration(_)));
}
