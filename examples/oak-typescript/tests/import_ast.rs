use oak_core::{Builder, ParseSession, SourceText};
use oak_typescript::ast::{ImportSpecifier, Statement};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};

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
