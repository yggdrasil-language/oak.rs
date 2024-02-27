use oak_core::{Lexer, Source, SourceText, lexer::NoLexerCache};
use oak_testing::lexing::LexerTester;
use oak_valkyrie::{ValkyrieLanguage, ValkyrieLexer, ValkyrieTokenType, lexer::ValkyrieKeywords};
use std::time::Duration;

#[test]
fn support_t_grammar_emits_template_tokens() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let lexer = ValkyrieLexer::new(&language);
    let source = SourceText::new("<% match arch %>");
    let mut cache = NoLexerCache;
    let output = lexer.lex(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let tokens = output.result.expect("lexer should succeed");
    let kinds = tokens.iter().map(|token| token.kind).collect::<Vec<_>>();
    assert!(kinds.contains(&ValkyrieTokenType::TemplateL), "{kinds:?}");
    assert!(kinds.contains(&ValkyrieTokenType::TemplateR), "{kinds:?}");
    assert!(!kinds.contains(&ValkyrieTokenType::TemplateText), "directive interior must not be `TemplateText`: {kinds:?}");
}

#[test]
fn support_t_grammar_gap_is_template_text() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let lexer = ValkyrieLexer::new(&language);
    let source = SourceText::new("<% loop i in items %>aaa<% end loop %>");
    let mut cache = NoLexerCache;
    let output = lexer.lex(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let tokens = output.result.expect("lexer should succeed");
    let text_spans = tokens.iter().filter(|token| token.kind == ValkyrieTokenType::TemplateText).map(|token| source.get_text_in(token.span)).collect::<Vec<_>>();
    assert_eq!(text_spans, vec!["aaa"]);
}

#[test]
fn support_t_grammar_valkyrie_source_in_gap_is_template_text() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let lexer = ValkyrieLexer::new(&language);
    let source = SourceText::new("<% loop i in j %>\nmicro fx() {}\n<% end loop %>");
    let mut cache = NoLexerCache;
    let output = lexer.lex(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let tokens = output.result.expect("lexer should succeed");
    let text = tokens.iter().find(|token| token.kind == ValkyrieTokenType::TemplateText).map(|token| source.get_text_in(token.span));
    assert!(text.is_some_and(|text| text.contains("micro fx()")));
    assert!(!tokens.iter().any(|token| token.kind == ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro)));
}

#[test]
fn support_t_grammar_end_in_directive_is_keyword() {
    let language = ValkyrieLanguage { support_t_grammar: true, ..ValkyrieLanguage::default() };
    let lexer = ValkyrieLexer::new(&language);
    let source = SourceText::new("<% end loop %>");
    let mut cache = NoLexerCache;
    let output = lexer.lex(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let tokens = output.result.expect("lexer should succeed");
    assert!(tokens.iter().any(|token| token.kind == ValkyrieTokenType::Keyword(ValkyrieKeywords::End)));
    assert!(!tokens.iter().any(|token| token.kind == ValkyrieTokenType::Identifier));
}

#[test]
fn support_t_grammar_disabled_rejects_template_source() {
    let language = ValkyrieLanguage { support_t_grammar: false, ..ValkyrieLanguage::default() };
    let lexer = ValkyrieLexer::new(&language);
    let source = SourceText::new("<% match arch %>");
    let mut cache = NoLexerCache;
    let output = lexer.lex(&source, &[], &mut cache);
    let tokens = output.result.expect("lexer should succeed");
    let kinds = tokens.iter().map(|token| token.kind).collect::<Vec<_>>();
    assert!(kinds.contains(&ValkyrieTokenType::Error), "{kinds:?}");
    assert!(!kinds.contains(&ValkyrieTokenType::TemplateL), "{kinds:?}");
    assert!(!kinds.contains(&ValkyrieTokenType::TemplateText), "{kinds:?}");
    assert!(!kinds.contains(&ValkyrieTokenType::TemplateR), "{kinds:?}");
}

#[test]
fn test_valkyrie_lexer() -> Result<(), oak_core::OakError> {
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests = here.join("tests/lexer");
    let config = ValkyrieLanguage::default();
    let lexer = ValkyrieLexer::new(&config);
    let tester = LexerTester::new(tests).with_extension("valkyrie").with_timeout(Duration::from_secs(5));
    tester.run_tests(&lexer)
}
