use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

#[test]
fn root_eof_does_not_spawn_empty_expr_stmt() {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    for src in ["return 40 + 2", "let x = 40", "40 + 2", "micro f() { return 1 }"] {
        let text = SourceText::new(src);
        let mut session = oak_core::ParseSession::<ValkyrieLanguage>::default();
        let out = builder.build(&text, &[], &mut session);
        assert!(out.result.is_ok(), "src={src:?} err={:?}", out.result.err());
        let root = out.result.unwrap();
        assert!(!root.items.is_empty(), "src={src:?}");
    }
}
