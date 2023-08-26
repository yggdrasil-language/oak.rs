use oak_core::{Builder, Source, SourceText};
use oak_valkyrie::{
    ValkyrieBuilder, ValkyrieLanguage, ValkyrieLexer, ValkyrieTokenType, ast::StatementNode,
};
use oak_core::Lexer;

#[test]
fn hash_is_line_comment_slash_is_not() {
    let lang = ValkyrieLanguage::default();
    let lexer = ValkyrieLexer::new(&lang);
    let src = SourceText::new("# hello\nmicro f() {}\n// not a comment\n");
    let mut session = oak_core::ParseSession::<ValkyrieLanguage>::default();
    let out = lexer.lex(&src, &[], &mut session);
    assert!(out.result.is_ok(), "lex err={:?}", out.result.err());
    let tokens = out.result.unwrap();
    let texts: Vec<_> = tokens
        .iter()
        .filter(|t| !matches!(t.kind, ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline))
        .map(|t| {
            let text = src.get_text_in(t.span.clone()).to_string();
            (format!("{:?}", t.kind), text)
        })
        .collect();
    assert!(
        texts.iter().any(|(k, t)| k == "LineComment" && t.starts_with('#')),
        "texts={texts:?}"
    );
    assert!(
        texts.iter().any(|(k, _)| k == "Slash"),
        "// must lex as Slash, texts={texts:?}"
    );
    assert!(
        !texts.iter().any(|(k, t)| k == "LineComment" && t.contains("//")),
        "// must not be LineComment, texts={texts:?}"
    );
}

#[test]
fn micro_with_typed_default_and_hash_comment_builds() {
    let lang = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&lang);
    let src = SourceText::new("# entry\nmicro on_update(x: f32 = 0.0) {}\n");
    let mut session = oak_core::ParseSession::<ValkyrieLanguage>::default();
    let out = builder.build(&src, &[], &mut session);
    assert!(out.result.is_ok(), "build failed: {:?}", out.result.err());
    let root = out.result.unwrap();
    assert_eq!(root.items.len(), 1, "items={:?}", root.items);
    match &root.items[0] {
        StatementNode::Micro(m) => {
            assert_eq!(m.name.name, "on_update");
            assert_eq!(m.params.len(), 1);
            assert_eq!(m.params[0].name.name, "x");
            assert!(m.params[0].ty.is_some());
            assert!(m.params[0].default.is_some());
        }
        other => panic!("expected Micro, got {other:?}"),
    }
}

#[test]
fn triple_slash_is_not_comment() {
    let lang = ValkyrieLanguage::default();
    let lexer = ValkyrieLexer::new(&lang);
    let src = SourceText::new("/// docs\nmicro f() {}\n");
    let mut session = oak_core::ParseSession::<ValkyrieLanguage>::default();
    let out = lexer.lex(&src, &[], &mut session);
    assert!(out.result.is_ok(), "lex err={:?}", out.result.err());
    let tokens = out.result.unwrap();
    let bad: Vec<_> = tokens
        .iter()
        .filter(|t| matches!(t.kind, ValkyrieTokenType::LineComment))
        .map(|t| src.get_text_in(t.span.clone()).to_string())
        .collect();
    assert!(bad.is_empty(), "/// must not be LineComment: {bad:?}");
}
