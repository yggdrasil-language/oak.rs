#![feature(new_range_api)]
use oak_core::{LexerState, source::Source};
use oak_testing::lexing::LexerTester;
use oak_wolfram::{WolframLanguage, WolframLexer};
use std::{path::Path, time::Duration};

#[test]
#[ignore = "manual baseline regeneration helper; do not run in CI"]
#[cfg(feature = "serde")]
fn generate_baseline() {
    use oak_core::{Lexer, ParseSession, SourceText, TokenType, source::Source};
    use oak_wolfram::{WolframLanguage, WolframLexer};
    use serde_json::json;
    use std::{fs, path::Path};

    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source_path = here.join("tests/lexer/basic.wl");
    let source_text = fs::read_to_string(source_path).expect("Failed to read source").replace("\r\n", "\n").replace('\r', "\n");
    let source = SourceText::new(source_text);
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);
    let mut cache = ParseSession::default();
    let result = lexer.lex(&source, &[], &mut cache);

    let tokens = result.result.expect("Lexing failed");
    // Match `LexerTester`: omit ignored trivia so the golden stays comparable.
    let token_data: Vec<_> = tokens
        .iter()
        .filter(|t| !t.kind.is_ignored())
        .map(|t| {
            let text = source.get_text_in(t.span.clone()).to_string();
            json!({
                "kind": format!("{:?}", t.kind),
                "text": text,
                "start": t.span.start,
                "end": t.span.end
            })
        })
        .collect();

    let output = json!({
        "success": true,
        "count": token_data.len(),
        "tokens": token_data,
        "errors": []
    });

    let output_path = here.join("tests/lexer/basic.wl.lexed.json");
    fs::write(output_path, serde_json::to_string_pretty(&output).unwrap()).expect("Failed to write baseline");

    println!("Baseline updated at {:?}", here.join("tests/lexer/basic.wl.lexed.json"));
}

#[test]
fn test_slot_n_and_message_name_tokens() {
    use oak_core::{Lexer, ParseSession, SourceText, TokenType};
    use oak_wolfram::WolframTokenType;

    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);
    let mut cache = ParseSession::default();

    let source = SourceText::new("#2 f::x ##3 ??Plus".to_string());
    let result = lexer.lex(&source, &[], &mut cache);
    let tokens = result.result.expect("lex");
    let kinds: Vec<_> = tokens
        .iter()
        .filter(|t| !t.kind.is_ignored())
        .map(|t| (format!("{:?}", t.kind), source.get_text_in(t.span.clone()).to_string()))
        .collect();

    assert!(
        kinds.iter().any(|(k, t)| k == "Slot" && t == "#2"),
        "expected Slot #2 token, got {kinds:?}"
    );
    assert!(
        kinds.iter().any(|(k, t)| k == "SlotSequence" && t == "##3"),
        "expected SlotSequence ##3 token, got {kinds:?}"
    );
    assert!(
        kinds.iter().any(|(k, _)| k == "MessageName"),
        "expected MessageName :: token, got {kinds:?}"
    );
    assert!(
        kinds.iter().any(|(k, _)| k == "DoubleQuestion"),
        "expected DoubleQuestion ?? token, got {kinds:?}"
    );
}

#[test]
#[cfg(feature = "serde")]
fn test_wolfram_lexer() {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);
    let test_runner = LexerTester::new(here.join("tests/lexer")).with_extension("wl").with_timeout(Duration::from_secs(5));
    match test_runner.run_tests::<WolframLanguage, _>(&lexer) {
        Ok(()) => println!("Wolfram lexer tests passed!"),
        Err(e) => panic!("Wolfram lexer tests failed: {}", e),
    }
}

#[test]
fn test_peek_behavior() {
    use oak_core::SourceText;
    use oak_wolfram::WolframLanguage;

    let source = SourceText::new("Module[{x}, x + 1]");
    let mut state = LexerState::<SourceText, WolframLanguage>::new(&source);

    println!("Initial state:");
    println!("Position: {}", state.get_position());
    println!("current(): {:?}", state.current());
    println!("peek(): {:?}", state.peek());

    println!("\nAfter advancing 1 char:");
    state.advance(1);
    println!("Position: {}", state.get_position());
    println!("current(): {:?}", state.current());
    println!("peek(): {:?}", state.peek());

    println!("\nAfter advancing 1 char:");
    state.advance(1);
    println!("Position: {}", state.get_position());
    println!("current(): {:?}", state.current());
    println!("peek(): {:?}", state.peek())
}

#[test]
fn test_wolfram_function_parsing() {
    use oak_core::{Lexer, SourceText};
    use oak_wolfram::{WolframLanguage, WolframLexer};

    let source = SourceText::new("Module[{x}, x + 1]");
    let language = WolframLanguage::default();
    let lexer = WolframLexer::new(&language);

    let mut cache = oak_core::ParseSession::<WolframLanguage>::default();
    let result = lexer.lex(&source, &[], &mut cache);

    println!("Testing Module[{{x}}, x + 1] parsing:");
    println!("Source code: '{}'", (&source).get_text_from(0));

    let tokens = result.result.expect("Lexing should succeed");
    assert!(!tokens.is_empty(), "Should parse at least one token");

    let first_token = &tokens[0];
    let source_ref = &source;
    let token_text = source_ref.get_text_in(first_token.span.clone());

    println!("First token: Kind={:?}, Text='{}', Position={}..{}", first_token.kind, token_text, first_token.span.start, first_token.span.end);

    assert_eq!(token_text, "Module", "Identifier should be parsed as Module");
    assert_eq!(first_token.span.start, 0, "Token should start at position 0");
    assert_eq!(first_token.span.end, 6, "Token should end at position 6");

    println!("✅ Module parsing test passed!")
}
