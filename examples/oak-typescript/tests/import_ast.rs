use oak_core::{Builder, ParseSession, SourceText};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ImportSpecifier, Statement},
};

#[test]
fn static_imports_build_module_specifier_and_bindings() {
    let source = SourceText::new(
        r#"
import { helper } from './lib';
import type { T } from '@pkg/types';
import * as ns from '../ns';
import def from './def';
export default class Page { x = 1; }
"#,
    );
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    let root = built.result.expect("build ok");

    let imports: Vec<_> = root
        .statements
        .iter()
        .filter_map(|s| match s {
            Statement::ImportDeclaration(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(imports.len(), 4, "imports={imports:?}");
    assert_eq!(imports[0].module_specifier, "./lib", "imp0={:?}", imports[0]);
    assert!(!imports[0].is_type_only);
    assert_eq!(imports[0].specifiers.len(), 1, "specifiers={:?}", imports[0].specifiers);
    match &imports[0].specifiers[0] {
        ImportSpecifier::Named { local, imported } => {
            assert_eq!(local, "helper");
            assert_eq!(imported, "helper");
        }
        other => panic!("expected Named helper, got {other:?}"),
    }

    assert_eq!(imports[1].module_specifier, "@pkg/types");
    assert!(imports[1].is_type_only);

    assert_eq!(imports[2].module_specifier, "../ns");
    assert!(matches!(&imports[2].specifiers[..], [ImportSpecifier::Namespace(n)] if n == "ns"));

    assert_eq!(imports[3].module_specifier, "./def");
    assert!(matches!(&imports[3].specifiers[..], [ImportSpecifier::Default(n)] if n == "def"));
}

#[test]
fn export_from_builds_source_and_specifiers() {
    let source = SourceText::new(
        r#"
export { a, b as c } from './mod';
export * from '../star';
export type { T } from '@pkg/types';
"#,
    );
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    let root = built.result.expect("build ok");

    let exports: Vec<_> = root
        .statements
        .iter()
        .filter_map(|s| match s {
            Statement::ExportDeclaration(d) => Some(d),
            _ => None,
        })
        .collect();
    assert_eq!(exports.len(), 3, "exports={exports:?}");

    assert_eq!(exports[0].source.as_deref(), Some("./mod"));
    assert_eq!(exports[0].specifiers.len(), 2);
    assert_eq!(exports[0].specifiers[0].local, "a");
    assert_eq!(exports[0].specifiers[0].exported, "a");
    assert_eq!(exports[0].specifiers[1].local, "b");
    assert_eq!(exports[0].specifiers[1].exported, "c");

    assert_eq!(exports[1].source.as_deref(), Some("../star"));
    assert!(exports[1].specifiers.is_empty());

    assert_eq!(exports[2].source.as_deref(), Some("@pkg/types"));
    assert!(exports[2].is_type_only);
}
