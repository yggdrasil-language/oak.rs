use oak_core::{Builder, ParseSession, Parser, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage, ValkyrieParser, ValkyrieRoot, ast::StatementNode};

fn parse_and_build(source: &str) -> ValkyrieRoot {
    let language = ValkyrieLanguage::default();
    let parser = ValkyrieParser::new(&language);
    let builder = ValkyrieBuilder::new(&language);
    let source = SourceText::new(source);
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let output = parser.parse(&source, &[], &mut session);
    assert!(output.result.is_ok(), "parser should succeed: {:?}", output.result.err());
    let built = builder.build(&source, &[], &mut ParseSession::default());
    built.result.expect("builder should succeed")
}

#[test]
fn sealed_class_modifier_is_ast_not_keyword() {
    let root = parse_and_build(
        r#"
sealed class Shape {
}
"#,
    );
    let class = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Class(class) => Some(class),
            _ => None,
        })
        .expect("expected class item");
    assert!(
        class.annotations.iter().any(|attribute| attribute.name.name == "sealed"),
        "sealed modifier should become annotation on class"
    );
}

#[test]
fn override_virtual_micro_modifiers_are_ast_not_keywords() {
    let root = parse_and_build(
        r#"
class Base {
    virtual micro area(self) -> i64 {
        return 0
    }
}

class Derived: Base {
    override micro area(self) -> i64 {
        return 1
    }
}
"#,
    );
    let base_method = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Class(class) if class.name.name == "Base" => class.methods.first(),
            _ => None,
        })
        .expect("expected base method");
    assert!(
        base_method.annotations.iter().any(|attribute| attribute.name.name == "virtual"),
        "virtual modifier should become annotation on micro"
    );

    let derived_method = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Class(class) if class.name.name == "Derived" => class.methods.first(),
            _ => None,
        })
        .expect("expected derived method");
    assert!(
        derived_method.annotations.iter().any(|attribute| attribute.name.name == "override"),
        "override modifier should become annotation on micro"
    );
}

#[test]
fn mut_let_binding_modifier_is_ast_not_keyword() {
    let root = parse_and_build(
        r#"
micro main() {
    let mut count = 0
}
"#,
    );
    let micro = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) => Some(micro),
            _ => None,
        })
        .expect("expected micro item");
    let let_stmt = micro
        .body
        .statements
        .iter()
        .find_map(|stmt| match stmt {
            oak_valkyrie::ast::Statement::Let(let_stmt) => Some(let_stmt),
            _ => None,
        })
        .expect("expected let statement");
    assert!(
        let_stmt.annotations.iter().any(|attribute| attribute.name.name == "mut"),
        "mut modifier should become annotation on let binding"
    );
}

#[test]
fn mut_parameter_modifier_is_ast_not_keyword() {
    let root = parse_and_build(
        r#"
micro increment(mut self) -> i64 {
    return 0
}
"#,
    );
    let param = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Micro(micro) => micro.params.first(),
            _ => None,
        })
        .expect("expected micro parameter");
    assert_eq!(param.name.name, "self");
    assert!(
        param.annotations.iter().any(|attribute| attribute.name.name == "mut"),
        "mut modifier should become annotation on parameter"
    );
}

#[test]
fn mut_struct_field_modifier_is_ast_not_keyword() {
    let root = parse_and_build(
        r#"
struct Counter {
    mut total: i64
}
"#,
    );
    let field = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Structure(structure) => structure.fields.first(),
            _ => None,
        })
        .expect("expected struct field");
    assert_eq!(field.name.name, "total");
    assert!(
        field.annotations.iter().any(|attribute| attribute.name.name == "mut"),
        "mut modifier should become annotation on field"
    );
}

#[test]
fn readonly_struct_field_modifier_is_ast_not_keyword() {
    let root = parse_and_build(
        r#"
struct Point {
    readonly x: f64
}
"#,
    );
    let field = root
        .items
        .iter()
        .find_map(|item| match item {
            StatementNode::Structure(structure) => structure.fields.first(),
            _ => None,
        })
        .expect("expected struct field");
    assert_eq!(field.name.name, "x");
    assert!(
        field.annotations.iter().any(|attribute| attribute.name.name == "readonly"),
        "readonly modifier should become annotation on field"
    );
}
