use oak_core::{Lexer, SourceText, TokenType};
use oak_mermaid::{MermaidLanguage, MermaidLexer, MermaidTokenType};
use oak_testing::lexing::LexerTester;
use std::{path::Path, time::Duration};

#[test]
fn test_mermaid_lexer() -> Result<(), oak_core::OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = MermaidLanguage::default();
    let lexer = MermaidLexer::new(&language);
    let test_runner = LexerTester::new(here.join("tests/lexer")).with_extension("mmd").with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&lexer)
}

#[test]
fn test_flowchart_token_kinds() -> Result<(), oak_core::OakError> {
    use MermaidTokenType::*;

    let language = MermaidLanguage::default();
    let lexer = MermaidLexer::new(&language);
    let source = SourceText::new("graph TD\n    A[Start] --> B{Decision}\n    B -->|yes| C[End]\n    C -.-> A\n    A --> D\n");
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = lexer.lex(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    let tokens = output.result.expect("lexing should succeed");
    let kinds: Vec<MermaidTokenType> = tokens.iter().filter(|token| !token.kind.is_ignored()).map(|token| token.kind).collect();

    assert_eq!(kinds, vec![Graph, Direction, Newline, Id, Label, Arrow, Id, Label, Newline, Id, Arrow, Label, Id, Label, Newline, Id, Arrow, Id, Newline, Id, Arrow, Id, Newline, Eof]);
    Ok(())
}
