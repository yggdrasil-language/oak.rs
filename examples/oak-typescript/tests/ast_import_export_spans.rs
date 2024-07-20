//! Module specifiers and import/export specifiers are real syntax tokens:
//! their spans slice back to the source text and `export * from` stays
//! distinguishable from `export {} from`.

use oak_core::{Builder, OakDiagnostics, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExportSpecifier, ImportSpecifier, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn root(source: &str) -> TypeScriptRoot {
    parse(source).result.expect("source should parse")
}

fn slice(source: &str, start: usize, end: usize) -> String {
    source[start..end].to_string()
}

fn import_decl(source: &str) -> oak_typescript::ast::ImportDeclaration {
    let root = root(source);
    let Statement::ImportDeclaration(declaration) = root.statements.into_iter().next().expect("one statement")
    else {
        panic!("expected an import declaration");
    };
    declaration
}

fn export_decl(source: &str) -> oak_typescript::ast::ExportDeclaration {
    let root = root(source);
    let Statement::ExportDeclaration(declaration) = root.statements.into_iter().next().expect("one statement")
    else {
        panic!("expected an export declaration");
    };
    declaration
}

#[test]
fn named_import_specifier_spans_slice_back_to_source() {
    let source = "import { A as B, type C } from './mod'";
    let declaration = import_decl(source);

    assert_eq!(declaration.module_specifier.value, "./mod");
    assert_eq!(slice(source, declaration.module_specifier.span.start, declaration.module_specifier.span.end), "'./mod'");

    let [ImportSpecifier::Named { local, imported, .. }, ImportSpecifier::Named { type_modifier, .. }] = declaration.specifiers.as_slice()
    else {
        panic!("expected two named specifiers, got {:?}", declaration.specifiers);
    };
    assert_eq!((local.as_str(), imported.as_str()), ("B", "A"));
    let first = declaration.specifiers[0].span();
    assert_eq!(slice(source, first.start, first.end), "A as B");
    let second = declaration.specifiers[1].span();
    assert_eq!(slice(source, second.start, second.end), "type C");
    assert!(type_modifier.is_some());
}

#[test]
fn default_and_namespace_import_specifier_spans_slice_back_to_source() {
    let default_source = "import def from \"d\"";
    let declaration = import_decl(default_source);
    let [ImportSpecifier::Default { local, .. }] = declaration.specifiers.as_slice()
    else {
        panic!("expected a default specifier, got {:?}", declaration.specifiers);
    };
    assert_eq!(local, "def");
    let span = declaration.specifiers[0].span();
    assert_eq!(slice(default_source, span.start, span.end), "def");
    assert_eq!(slice(default_source, declaration.module_specifier.span.start, declaration.module_specifier.span.end), "\"d\"");

    let ns_source = "import * as ns from 'n'";
    let declaration = import_decl(ns_source);
    let [ImportSpecifier::Namespace { local, .. }] = declaration.specifiers.as_slice()
    else {
        panic!("expected a namespace specifier, got {:?}", declaration.specifiers);
    };
    assert_eq!(local, "ns");
    let span = declaration.specifiers[0].span();
    assert_eq!(slice(ns_source, span.start, span.end), "* as ns");
}

#[test]
fn side_effect_import_records_specifier_span() {
    let source = "import './x';";
    let declaration = import_decl(source);
    assert!(declaration.specifiers.is_empty());
    assert_eq!(declaration.module_specifier.value, "./x");
    assert_eq!(slice(source, declaration.module_specifier.span.start, declaration.module_specifier.span.end), "'./x'");
}

#[test]
fn export_specifier_and_source_spans_slice_back_to_source() {
    let source = "export { A, B as C } from './m'";
    let declaration = export_decl(source);

    let ExportSpecifier::Named { local, exported, .. } = &declaration.specifiers[0]
    else {
        panic!("expected a named specifier");
    };
    assert_eq!((local.as_str(), exported.as_str()), ("A", "A"));
    let first = declaration.specifiers[0].span();
    assert_eq!(slice(source, first.start, first.end), "A");
    let second = declaration.specifiers[1].span();
    assert_eq!(slice(source, second.start, second.end), "B as C");

    let source_specifier = declaration.source.as_ref().expect("export source");
    assert_eq!(source_specifier.value, "./m");
    assert_eq!(slice(source, source_specifier.span.start, source_specifier.span.end), "'./m'");
}

#[test]
fn export_star_is_distinguishable_from_empty_clause() {
    let star = export_decl("export * from 'all'");
    let empty = export_decl("export {} from 'none'");

    let [ExportSpecifier::All { .. }] = star.specifiers.as_slice()
    else {
        panic!("`export * from` must record a star specifier, got {:?}", star.specifiers);
    };
    assert_eq!(slice("export * from 'all'", star.specifiers[0].span().start, star.specifiers[0].span().end), "*");
    assert!(empty.specifiers.is_empty(), "`export {{}} from` must keep an empty specifier list");
    assert_eq!(star.source.as_ref().expect("star source").value, "all");
    assert_eq!(empty.source.as_ref().expect("empty source").value, "none");
}

#[test]
fn export_star_as_namespace_records_alias_and_span() {
    let source = "export * as ns from 'm'";
    let declaration = export_decl(source);
    let [ExportSpecifier::Namespace { exported, .. }] = declaration.specifiers.as_slice()
    else {
        panic!("expected a namespace specifier, got {:?}", declaration.specifiers);
    };
    assert_eq!(exported, "ns");
    let span = declaration.specifiers[0].span();
    assert_eq!(slice(source, span.start, span.end), "* as ns");
}
