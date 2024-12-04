//! A property's declared name must survive an initializer that is itself an
//! identifier. `rows: Row[] = SEED` used to report the field name `SEED`,
//! because the builder overwrote `name` with every identifier child. VMZ then
//! allocated a reactive state slot named after the module constant and dropped
//! the real field, so the component lost `rows` entirely.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn class_members(source: &str) -> Vec<ClassMember> {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    root.statements
        .into_iter()
        .find_map(|statement| match statement {
            Statement::ClassDeclaration(declaration) => Some(declaration.body),
            _ => None,
        })
        .unwrap_or_else(|| panic!("expected class declaration in {source}"))
}

fn property_names(source: &str) -> Vec<String> {
    class_members(source)
        .into_iter()
        .filter_map(|member| match member {
            ClassMember::Property { name, .. } => Some(name),
            _ => None,
        })
        .collect()
}

#[test]
fn typed_property_keeps_its_name_when_initializer_is_an_identifier() {
    let names = property_names("type Row = { id: string };\nclass C {\n  rows: Row[] = SEED;\n  label = 'x';\n}");
    assert_eq!(names, vec!["rows".to_string(), "label".to_string()]);
}

#[test]
fn untyped_property_keeps_its_name_when_initializer_is_an_identifier() {
    let names = property_names("class C {\n  rows = SEED;\n}");
    assert_eq!(names, vec!["rows".to_string()]);
}

#[test]
fn property_initializer_is_still_captured() {
    let members = class_members("class C {\n  rows: Row[] = SEED;\n}");
    let [ClassMember::Property { name, initializer, ty, .. }] = members.as_slice()
    else {
        panic!("expected one property, got {members:?}")
    };
    assert_eq!(name, "rows");
    assert!(initializer.is_some(), "initializer must be kept (has_ty={})", ty.is_some());
}
