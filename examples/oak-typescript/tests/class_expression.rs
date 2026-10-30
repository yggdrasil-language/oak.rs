//! Classes in expression position (`var X = class { … }`, `f(class {})`,
//! `new class {}`) and the anonymous `export default class { … }` form.
//! The parser used to reject `class` in `primary()` and required a name in
//! `parse_class_declaration_content`, so the class body leaked out as a
//! top-level block statement and anonymous default exports failed outright.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, ExpressionKind, Statement, TypeAnnotation, TypeScriptRoot},
    formatter::{FormatOptions, format_source},
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

fn variable_initializer(source: &str) -> ExpressionKind {
    let root = parsed_root(source);
    let Some(Statement::VariableDeclaration(declaration)) = root.statements.first().cloned() else {
        panic!("expected variable declaration in {source}");
    };
    let value = declaration.value().expect("initializer");
    *value.kind.clone()
}

fn class_expression(kind: ExpressionKind) -> (Option<String>, Option<String>, Vec<ClassMember>) {
    match kind {
        ExpressionKind::ClassExpression { name, extends, body, .. } => {
            let extends_text = match extends {
                Some(TypeAnnotation::Reference { name, .. }) => Some(name),
                Some(other) => panic!("unexpected extends shape {other:?}"),
                None => None,
            };
            (name, extends_text, body)
        }
        other => panic!("expected class expression, got {other:?}"),
    }
}

#[test]
fn anonymous_class_expression_in_variable_initializer() {
    let (name, extends, body) = class_expression(variable_initializer("var X = class { };"));
    assert_eq!(name, None);
    assert_eq!(extends, None);
    assert!(body.is_empty());
}

#[test]
fn class_expression_with_fields_and_constructor() {
    let (name, _, body) = class_expression(variable_initializer("var X = class { a = 1; constructor(v) { this.a = v; } b = f(this.a); };"));
    assert_eq!(name, None);
    assert_eq!(body.len(), 3);
    let ClassMember::Property { name, .. } = &body[0] else { panic!("expected property `a`, got {:?}", body[0]) };
    assert_eq!(name, "a");
    let ClassMember::Method { name, params, .. } = &body[1] else { panic!("expected constructor, got {:?}", body[1]) };
    assert_eq!(name, "constructor");
    assert_eq!(params.len(), 1);
    let ClassMember::Property { name, initializer, .. } = &body[2] else { panic!("expected property `b`, got {:?}", body[2]) };
    assert_eq!(name, "b");
    assert!(initializer.is_some(), "field initializer `f(this.a)` must survive");
}

#[test]
fn named_class_expression_with_extends_and_static_field() {
    let (name, extends, body) = class_expression(variable_initializer("const C = class Named extends Base { static s = 1; m() { } };"));
    assert_eq!(name.as_deref(), Some("Named"));
    assert_eq!(extends.as_deref(), Some("Base"));
    assert_eq!(body.len(), 2);
    let ClassMember::Property { name, .. } = &body[0] else { panic!("expected property `s`, got {:?}", body[0]) };
    assert_eq!(name, "s");
    assert!(body[0].is_static());
    let ClassMember::Method { name, .. } = &body[1] else { panic!("expected method `m`, got {:?}", body[1]) };
    assert_eq!(name, "m");
}

#[test]
fn class_expression_as_call_argument() {
    let root = parsed_root("f(class { });");
    let Some(Statement::ExpressionStatement(statement)) = root.statements.first().cloned() else {
        panic!("expected expression statement");
    };
    let ExpressionKind::CallExpression { args, .. } = *statement.expression.kind else {
        panic!("expected call expression");
    };
    assert_eq!(args.len(), 1);
    let (name, _, body) = class_expression(*args[0].kind.clone());
    assert_eq!(name, None);
    assert!(body.is_empty());
}

#[test]
fn vscode_textmate_style_class_expression_parses_clean() {
    let (name, _, body) = class_expression(variable_initializer("var RelativeReference = class { constructor(ruleName) { this.ruleName = ruleName; } kind = 2; };"));
    assert_eq!(name, None);
    assert_eq!(body.len(), 2);
    assert!(matches!(&body[0], ClassMember::Method { name, .. } if name == "constructor"));
    assert!(matches!(&body[1], ClassMember::Property { name, initializer: Some(_), .. } if name == "kind"));
}

#[test]
fn anonymous_default_export_class() {
    let root = parsed_root("export default class { c = 3; }");
    let Some(Statement::ExportDeclaration(export)) = root.statements.first().cloned() else {
        panic!("expected export declaration");
    };
    assert!(export.is_default());
    let inner = export.declaration.expect("default payload");
    let Statement::ClassDeclaration(declaration) = *inner else {
        panic!("expected class declaration, got {inner:?}");
    };
    assert_eq!(declaration.name, "");
    assert_eq!(declaration.body.len(), 1);
    assert!(matches!(&declaration.body[0], ClassMember::Property { name, initializer: Some(_), .. } if name == "c"));
}

#[test]
fn named_default_export_class_keeps_its_name() {
    let root = parsed_root("export default class Named { }");
    let Some(Statement::ExportDeclaration(export)) = root.statements.first().cloned() else {
        panic!("expected export declaration");
    };
    let inner = export.declaration.expect("default payload");
    let Statement::ClassDeclaration(declaration) = *inner else {
        panic!("expected class declaration, got {inner:?}");
    };
    assert_eq!(declaration.name, "Named");
}

#[test]
fn named_class_declaration_regression() {
    let root = parsed_root("class Y { a = 1; }");
    let Some(Statement::ClassDeclaration(declaration)) = root.statements.first().cloned() else {
        panic!("expected class declaration");
    };
    assert_eq!(declaration.name, "Y");
    assert_eq!(declaration.body.len(), 1);
    assert!(matches!(&declaration.body[0], ClassMember::Property { name, initializer: Some(_), .. } if name == "a"));
}

#[test]
fn class_expression_survives_the_formatter_round_trip() {
    for source in ["var X = class { };\n", "var X = class { a = 1; constructor(v) { this.a = v; } };\n", "const C = class Named extends Base { static s = 1; m() { } };\n", "export default class { c = 3; }\n"] {
        let options = FormatOptions::default();
        let output = format_source(source, &options).expect("format");
        assert!(output.contains("class"), "class keyword lost: {output:?}");
        assert_eq!(format_source(&output, &options).expect("idempotent"), output, "not idempotent: {source:?}");
    }
}
