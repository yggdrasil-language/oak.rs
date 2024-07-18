//! AST fields must record real syntax nodes with spans. `is_*` facts are derived
//! methods, never inferred booleans.

use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, ImportSpecifier, ModifierKind, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn root(source: &str) -> TypeScriptRoot {
    parse(source).result.expect("source should parse")
}

fn module_text(source: &str, start: usize, end: usize) -> String {
    source[start..end].to_string()
}

#[test]
fn mixed_named_import_is_not_declaration_level_type_only() {
    let source = "import { REGISTRY, type IconDef } from './icons'";
    let root = root(source);
    let Statement::ImportDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected an import declaration");
    };
    assert!(!declaration.is_type_only(), "a per-binding `type` keyword must not mark the whole import");
    assert!(declaration.modifiers.is_empty(), "no declaration-level modifier was written");

    let [ImportSpecifier::Named { local: registry, type_modifier: registry_type, .. }, ImportSpecifier::Named { local: icon_def, type_modifier: icon_def_type, .. }] = declaration.specifiers.as_slice()
    else {
        panic!("expected two named bindings, got {:?}", declaration.specifiers);
    };
    assert_eq!(registry, "REGISTRY");
    assert_eq!(icon_def, "IconDef");
    assert!(registry_type.is_none());
    let marker = icon_def_type.as_ref().expect("`type` keyword must be recorded on the specifier");
    assert_eq!(marker.kind, ModifierKind::Type);
    assert_eq!(module_text(source, marker.span.start, marker.span.end), "type");
    assert!(declaration.specifiers[1].is_inline_type_only());
    assert!(!declaration.specifiers[0].is_inline_type_only());
}

#[test]
fn import_type_records_declaration_level_modifier() {
    let source = "import type { A } from './a'";
    let root = root(source);
    let Statement::ImportDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected an import declaration");
    };
    assert!(declaration.is_type_only());
    assert_eq!(declaration.modifiers.len(), 1);
    assert_eq!(module_text(source, declaration.modifiers[0].span.start, declaration.modifiers[0].span.end), "type");
    let [ImportSpecifier::Named { type_modifier, .. }] = declaration.specifiers.as_slice()
    else {
        panic!("expected one named binding");
    };
    assert!(type_modifier.is_none(), "declaration-level `type` is not a per-binding keyword");
}

#[test]
fn export_specifiers_keep_inline_type_modifiers() {
    let source = "export { A, type B } from './b'";
    let root = root(source);
    let Statement::ExportDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected an export declaration");
    };
    assert!(!declaration.is_type_only());
    assert!(declaration.modifiers.is_empty());
    assert_eq!(declaration.specifiers.len(), 2);
    assert!(!declaration.specifiers[0].is_inline_type_only());
    assert!(declaration.specifiers[1].is_inline_type_only());
    let marker = declaration.specifiers[1].type_modifier.as_ref().expect("inline `type` must be recorded");
    assert_eq!(module_text(source, marker.span.start, marker.span.end), "type");
    assert_eq!(declaration.source.as_deref(), Some("./b"));
}

#[test]
fn export_type_records_declaration_level_modifier() {
    let source = "export type { A } from './a'";
    let root = root(source);
    let Statement::ExportDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected an export declaration");
    };
    assert!(declaration.is_type_only());
    assert!(!declaration.is_default());
    assert_eq!(declaration.specifiers.len(), 1);
    assert!(!declaration.specifiers[0].is_inline_type_only());
}

#[test]
fn export_default_records_default_modifier() {
    let source = "export default class X {}";
    let root = root(source);
    let Statement::ExportDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected an export declaration");
    };
    assert!(declaration.is_default());
    assert!(!declaration.is_type_only());
    assert_eq!(module_text(source, declaration.modifiers[0].span.start, declaration.modifiers[0].span.end), "default");
    assert!(declaration.specifiers.is_empty());
    assert!(declaration.source.is_none());
}

#[test]
fn declare_and_abstract_are_recorded_keyword_tokens() {
    let declare_source = "declare const x = 1";
    let declare_root = root(declare_source);
    let Statement::VariableDeclaration(declaration) = &declare_root.statements[0]
    else {
        panic!("expected a variable declaration");
    };
    assert!(declaration.is_declare());
    assert_eq!(declaration.modifiers.len(), 1);
    assert_eq!(declaration.modifiers[0].kind, ModifierKind::Declare);
    assert_eq!(module_text(declare_source, declaration.modifiers[0].span.start, declaration.modifiers[0].span.end), "declare");

    let abstract_source = "abstract class X {}";
    let abstract_root = root(abstract_source);
    let Statement::ClassDeclaration(declaration) = &abstract_root.statements[0]
    else {
        panic!("expected a class declaration");
    };
    assert!(declaration.is_abstract());
    assert!(!declaration.is_declare());
    assert_eq!(declaration.modifiers.len(), 1);
    assert_eq!(declaration.modifiers[0].kind, ModifierKind::Abstract);
}

#[test]
fn class_members_record_modifier_tokens_and_optional_marker() {
    let source = "class X { static readonly a = 1; private get b() {} }";
    let root = root(source);
    let Statement::ClassDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected a class declaration");
    };
    let [ClassMember::Property { modifiers, optional_marker, .. }, ClassMember::Method { modifiers: method_modifiers, .. }] = declaration.body.as_slice()
    else {
        panic!("expected one property and one accessor, got {:?}", declaration.body);
    };
    assert_eq!(modifiers.iter().map(|m| m.kind).collect::<Vec<_>>(), vec![ModifierKind::Static, ModifierKind::Readonly]);
    assert_eq!(module_text(source, modifiers[0].span.start, modifiers[0].span.end), "static");
    assert_eq!(module_text(source, modifiers[1].span.start, modifiers[1].span.end), "readonly");
    assert!(optional_marker.is_none());
    assert!(declaration.body[0].is_static());
    assert!(declaration.body[0].is_readonly());
    assert!(!declaration.body[0].is_optional());

    assert_eq!(method_modifiers.iter().map(|m| m.kind).collect::<Vec<_>>(), vec![ModifierKind::Get]);
    assert!(declaration.body[1].is_getter());
    assert!(!declaration.body[1].is_setter());
}

#[test]
fn function_param_optional_marker_is_a_real_token() {
    let source = "function f(a?: number) {}";
    let root = root(source);
    let Statement::FunctionDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected a function declaration");
    };
    let param = declaration.params.first().expect("one parameter expected");
    assert!(param.is_optional());
    let marker = param.optional_marker.as_ref().expect("`?` must be recorded");
    assert_eq!(module_text(source, marker.span.start, marker.span.end), "?");
}
