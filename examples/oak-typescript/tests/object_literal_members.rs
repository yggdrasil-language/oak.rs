//! Object literal member construction: every member must be retained, in source
//! order, with spans that stop exactly at the last byte of the member.

use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Expression, ExpressionKind, ObjectProperty, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn variable_value(source: &str) -> Expression {
    let output = parse(source);
    let root = output.result.expect("source should parse");
    let statement = root.statements.first().expect("one statement expected");
    let Statement::VariableDeclaration(declaration) = statement
    else {
        panic!("expected a variable declaration, got {statement:?}");
    };
    declaration.value().cloned().expect("expected an initialized variable declaration")
}

fn properties(source: &str) -> Vec<ObjectProperty> {
    let value = variable_value(source);
    let ExpressionKind::ObjectLiteral { properties } = value.kind.as_ref()
    else {
        panic!("expected an object literal");
    };
    properties.clone()
}

fn property_text(source: &str, start: usize, end: usize) -> String {
    source[start..end].to_string()
}

fn property_parts(source: &str, property: &ObjectProperty) -> (String, bool, (usize, usize), String) {
    match property {
        ObjectProperty::Property { name, shorthand, span, .. } => (name.clone(), *shorthand, (span.start, span.end), property_text(source, span.start, span.end)),
        ObjectProperty::Spread(_) => panic!("expected a property, got a spread"),
    }
}

fn value_kind(property: &ObjectProperty) -> ExpressionKind {
    match property {
        ObjectProperty::Property { value, .. } => value.kind.as_ref().clone(),
        ObjectProperty::Spread(value) => value.kind.as_ref().clone(),
    }
}

fn value_identifier(property: &ObjectProperty) -> String {
    match value_kind(property) {
        ExpressionKind::Identifier(name) => name,
        other => panic!("expected an identifier value, got {other:?}"),
    }
}

fn value_string(property: &ObjectProperty) -> String {
    match value_kind(property) {
        ExpressionKind::StringLiteral(text) => text,
        other => panic!("expected a string literal value, got {other:?}"),
    }
}

fn value_text(source: &str, property: &ObjectProperty) -> String {
    match property {
        ObjectProperty::Property { value, .. } => property_text(source, value.span.start, value.span.end),
        ObjectProperty::Spread(_) => panic!("expected a property, got a spread"),
    }
}

#[test]
fn object_literal_retains_all_properties_with_exact_spans() {
    let source = "const x = { a: 1, b: 2 };";
    let literal = variable_value(source);
    assert_eq!(property_text(source, literal.span.start, literal.span.end), "{ a: 1, b: 2 }");
    assert_eq!(literal.span.end, 24);

    let properties = properties(source);
    assert_eq!(properties.len(), 2);
    assert_eq!(property_parts(source, &properties[0]), ("a".to_string(), false, (12, 16), "a: 1".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("b".to_string(), false, (18, 22), "b: 2".to_string()));
    assert_eq!(value_text(source, &properties[0]), "1");
    assert_eq!(value_text(source, &properties[1]), "2");
}

#[test]
fn object_literal_retains_properties_before_a_trailing_comma() {
    let source = "const x = { a: 1, b: 2, };";
    let properties = properties(source);
    assert_eq!(properties.len(), 2);
    assert_eq!(property_parts(source, &properties[0]), ("a".to_string(), false, (12, 16), "a: 1".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("b".to_string(), false, (18, 22), "b: 2".to_string()));
}

#[test]
fn object_literal_retains_properties_split_over_lines() {
    let source = "const x = {\n  a: 1,\n  b: 2,\n};";
    let properties = properties(source);
    assert_eq!(properties.len(), 2);
    assert_eq!(property_parts(source, &properties[0]), ("a".to_string(), false, (14, 18), "a: 1".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("b".to_string(), false, (22, 26), "b: 2".to_string()));
}

#[test]
fn object_literal_keeps_identifier_valued_properties() {
    let source = "const x = { name: label, type: \"text\", count: total };";
    let properties = properties(source);
    assert_eq!(properties.len(), 3);
    assert_eq!(property_parts(source, &properties[0]), ("name".to_string(), false, (12, 23), "name: label".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("type".to_string(), false, (25, 37), "type: \"text\"".to_string()));
    assert_eq!(property_parts(source, &properties[2]), ("count".to_string(), false, (39, 51), "count: total".to_string()));
    assert_eq!(value_identifier(&properties[0]), "label");
    assert_eq!(value_string(&properties[1]), "text");
    assert_eq!(value_identifier(&properties[2]), "total");
}

#[test]
fn object_literal_keeps_reserved_word_property_names() {
    let source = "const x = { type: \"text\", default: false, size: 2 };";
    let properties = properties(source);
    assert_eq!(properties.len(), 3);
    assert_eq!(property_parts(source, &properties[0]), ("type".to_string(), false, (12, 24), "type: \"text\"".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("default".to_string(), false, (26, 40), "default: false".to_string()));
    assert_eq!(property_parts(source, &properties[2]), ("size".to_string(), false, (42, 49), "size: 2".to_string()));
}

#[test]
fn object_literal_keeps_shorthand_property_names_and_spans() {
    let source = "const x = { a, b };";
    let properties = properties(source);
    assert_eq!(properties.len(), 2);
    assert_eq!(property_parts(source, &properties[0]), ("a".to_string(), true, (12, 13), "a".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("b".to_string(), true, (15, 16), "b".to_string()));
    assert_eq!(value_identifier(&properties[0]), "a");
    assert_eq!(value_identifier(&properties[1]), "b");
}

#[test]
fn object_literal_keeps_computed_and_plain_properties() {
    let source = "const x = { [key]: 1, plain: 2 };";
    let properties = properties(source);
    assert_eq!(properties.len(), 2);
    assert_eq!(property_parts(source, &properties[0]), ("key".to_string(), false, (12, 20), "[key]: 1".to_string()));
    assert_eq!(property_parts(source, &properties[1]), ("plain".to_string(), false, (22, 30), "plain: 2".to_string()));
}

#[test]
fn object_literal_keeps_spread_and_following_properties() {
    let source = "const x = { ...base, a: 1 };";
    let properties = properties(source);
    assert_eq!(properties.len(), 2);
    assert_eq!(value_identifier(&properties[0]), "base");
    assert_eq!(property_parts(source, &properties[1]), ("a".to_string(), false, (21, 25), "a: 1".to_string()));
}
