//! `ClassDeclaration` heritage facts. The parser keeps `extends` / `implements`
//! flat under the declaration node (no `HeritageClause` wrapper), so the
//! builder used to leave `ClassDeclaration::extends` `None` while the
//! class-expression path resolved the same flat children correctly. These
//! tests pin the declaration side to the expression side.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Statement, TypeAnnotation, TypeScriptRoot},
};

fn root_of(source: &str, erase_types: bool) -> TypeScriptRoot {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let output = TypeScriptBuilder::new(&language).with_type_erasure(erase_types).build(&SourceText::new(source), &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    output.result.expect("source should build")
}

fn class_declaration(source: &str) -> oak_typescript::ast::ClassDeclaration {
    let root = root_of(source, false);
    let Some(Statement::ClassDeclaration(declaration)) = root.statements.first().cloned() else {
        panic!("expected class declaration in {source}");
    };
    declaration
}

fn reference_name(ty: &TypeAnnotation) -> &str {
    match ty {
        TypeAnnotation::Reference { name, .. } => name,
        other => panic!("expected type reference, got {other:?}"),
    }
}

#[test]
fn extends_clause_resolves_to_a_reference() {
    let declaration = class_declaration("class Derived extends Base { }");
    assert_eq!(declaration.name, "Derived");
    let extends = declaration.extends.expect("`extends Base` must produce a heritage fact");
    assert_eq!(reference_name(&extends), "Base");
    assert!(declaration.implements.is_empty());
}

#[test]
fn implements_clause_resolves_to_references() {
    let declaration = class_declaration("class Impl implements First, Second { }");
    assert!(declaration.extends.is_none(), "expected no extends fact, got {:?}", declaration.extends);
    let names: Vec<&str> = declaration.implements.iter().map(reference_name).collect();
    assert_eq!(names, vec!["First", "Second"]);
}

#[test]
fn extends_and_implements_clauses_resolve_together() {
    let declaration = class_declaration("class Both extends Parent implements Left, Right { }");
    assert_eq!(reference_name(&declaration.extends.expect("extends fact")), "Parent");
    let names: Vec<&str> = declaration.implements.iter().map(reference_name).collect();
    assert_eq!(names, vec!["Left", "Right"]);
}

#[test]
fn heritage_name_slices_back_to_the_source() {
    let source = "class Derived extends Base { }";
    let declaration = class_declaration(source);
    let extends = declaration.extends.expect("extends fact");
    let name = reference_name(&extends);
    // The heritage reference text appears verbatim inside the declaration span.
    let declaration_text = &source[declaration.span.clone()];
    assert!(declaration_text.contains(name), "declaration text {declaration_text:?} must contain heritage name {name:?}");
    let after_extends = declaration_text.split("extends ").nth(1).expect("`extends ` prefix");
    assert!(after_extends.starts_with(name), "heritage name {name:?} must start right after `extends `");
}

#[test]
fn dotted_and_generic_heritage_keep_the_expression_shape() {
    // Mirrors the class-expression builder: the whole `TypeReference` node text
    // is the reference name, dot chains and type arguments included.
    let dotted = class_declaration("class D extends ns.Base { }");
    assert_eq!(reference_name(&dotted.extends.expect("dotted heritage fact")), "ns.Base");
    let generic = class_declaration("class D extends Base<number> { }");
    assert_eq!(reference_name(&generic.extends.expect("generic heritage fact")), "Base<number>");
}

#[test]
fn type_erasure_keeps_extends_and_drops_implements() {
    let root = root_of("class D extends Base implements I { }", true);
    let Some(Statement::ClassDeclaration(declaration)) = root.statements.first().cloned() else {
        panic!("expected class declaration");
    };
    assert_eq!(reference_name(&declaration.extends.expect("extends survives erasure")), "Base");
    assert!(declaration.implements.is_empty(), "implements must be erased with the types");
}

#[test]
fn exported_and_default_class_declarations_resolve_heritage() {
    let root = root_of("export class Exported extends Base { }", false);
    let Some(Statement::ExportDeclaration(export)) = root.statements.first().cloned() else {
        panic!("expected export declaration");
    };
    let inner = export.declaration.expect("exported payload");
    let Statement::ClassDeclaration(declaration) = *inner else {
        panic!("expected class declaration, got {inner:?}");
    };
    assert_eq!(reference_name(&declaration.extends.expect("extends fact")), "Base");

    let root = root_of("export default class Named extends Other { }", false);
    let Some(Statement::ExportDeclaration(export)) = root.statements.first().cloned() else {
        panic!("expected export declaration");
    };
    let inner = export.declaration.expect("default payload");
    let Statement::ClassDeclaration(declaration) = *inner else {
        panic!("expected class declaration, got {inner:?}");
    };
    assert_eq!(declaration.name, "Named");
    assert_eq!(reference_name(&declaration.extends.expect("extends fact")), "Other");
}

#[test]
fn abstract_class_heritage_resolves() {
    let declaration = class_declaration("abstract class Shape extends Geometry { }");
    assert!(declaration.is_abstract());
    assert_eq!(reference_name(&declaration.extends.expect("extends fact")), "Geometry");
}

#[test]
fn class_without_heritage_stays_empty() {
    let declaration = class_declaration("class Plain { }");
    assert!(declaration.extends.is_none(), "expected no extends fact, got {:?}", declaration.extends);
    assert!(declaration.implements.is_empty());
}
