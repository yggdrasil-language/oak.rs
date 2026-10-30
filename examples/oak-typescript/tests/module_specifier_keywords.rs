//! Module specifier names may be keywords: `export { default } from './m'`
//! and `export { default as x } from './m'` must record the keyword name and
//! its alias instead of reporting an expected `RightBrace`.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExportDeclaration, ExportSpecifier, ImportSpecifier, Statement, TypeScriptRoot},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn export_decl(source: &str) -> ExportDeclaration {
    let root = build(source);
    let Statement::ExportDeclaration(decl) = root.statements.into_iter().next().expect("one statement")
    else {
        panic!("expected an export declaration");
    };
    decl
}

#[test]
fn export_default_specifier_from_module() {
    let source = "export { default } from './m.js'";
    let declaration = export_decl(source);

    let ExportSpecifier::Named { local, exported, .. } = &declaration.specifiers[0]
    else {
        panic!("expected a named specifier: {:?}", declaration.specifiers)
    };
    assert_eq!((local.as_str(), exported.as_str()), ("default", "default"));
    let span = declaration.specifiers[0].span();
    assert_eq!(&source[span.start..span.end], "default");
    assert_eq!(declaration.source.as_ref().expect("export source").value, "./m.js");
}

#[test]
fn export_default_as_alias_from_module() {
    let source = "export { default as x } from './m.js'";
    let declaration = export_decl(source);

    let ExportSpecifier::Named { local, exported, .. } = &declaration.specifiers[0]
    else {
        panic!("expected a named specifier: {:?}", declaration.specifiers)
    };
    assert_eq!((local.as_str(), exported.as_str()), ("default", "x"));
    let span = declaration.specifiers[0].span();
    assert_eq!(&source[span.start..span.end], "default as x");
    assert_eq!(declaration.source.as_ref().expect("export source").value, "./m.js");
}

#[test]
fn import_default_keyword_specifier_parses() {
    let source = "import { default as d } from './m.js'";
    let root = build(source);
    let Statement::ImportDeclaration(declaration) = &root.statements[0]
    else {
        panic!("expected an import declaration: {:?}", root.statements)
    };
    let ImportSpecifier::Named { local, imported, .. } = &declaration.specifiers[0]
    else {
        panic!("expected a named specifier: {:?}", declaration.specifiers)
    };
    assert_eq!((imported.as_str(), local.as_str()), ("default", "d"));
    assert_eq!(declaration.module_specifier.value, "./m.js");
}
