use oak_core::{Lexer, Source, SourceText, TokenType, parser::session::ParseSession};
use oak_sql::{SqlLanguage, SqlLexer, SqlTokenType};
use oak_testing::lexing::LexerTester;
use std::{path::Path, time::Duration};

#[test]
fn test_sql_lexer() -> Result<(), oak_core::OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = SqlLanguage::default();
    let lexer = SqlLexer::new(&language);
    let test_runner = LexerTester::new(here.join("tests/lexer")).with_extension("sql").with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&lexer)
}

/// Lexes `input` and returns the significant tokens with their source text.
fn lex(input: &str) -> Vec<(SqlTokenType, String)> {
    let language = SqlLanguage::default();
    let lexer = SqlLexer::new(&language);
    let source = SourceText::new(input);
    let mut cache = ParseSession::<SqlLanguage>::default();
    let output = lexer.lex(&source, &[], &mut cache);
    let tokens = output.result.unwrap_or_else(|error| panic!("{input}: {error:?}"));
    tokens.iter().filter(|token| !token.kind.is_ignored() && token.kind != SqlTokenType::Eof).map(|token| (token.kind, source.get_text_in(token.span.clone()).into_owned())).collect()
}

#[test]
fn test_sql_system_variable_is_one_identifier() {
    let tokens = lex("SELECT @@session.sql_mode");
    assert_eq!(
        tokens,
        vec![(SqlTokenType::Select, "SELECT".to_string()), (SqlTokenType::Identifier_, "@@session.sql_mode".to_string())]
    );

    let tokens = lex("@@max_connections");
    assert_eq!(tokens, vec![(SqlTokenType::Identifier_, "@@max_connections".to_string())]);
}

#[test]
fn test_sql_lone_at_sign_is_an_error_token() {
    let tokens = lex("@name");
    assert_eq!(tokens, vec![(SqlTokenType::Error, "@".to_string()), (SqlTokenType::Identifier_, "name".to_string())]);

    let tokens = lex("@@");
    assert_eq!(tokens, vec![(SqlTokenType::Error, "@@".to_string())]);
}

#[test]
fn test_sql_predicate_keywords_are_keywords() {
    let tokens = lex("a LIKE b AND c BETWEEN 1 AND 2 OR d IN (1) AND e IS NULL");
    let kinds = tokens.iter().map(|(kind, text)| (*kind, text.as_str())).collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            (SqlTokenType::Identifier_, "a"),
            (SqlTokenType::Like, "LIKE"),
            (SqlTokenType::Identifier_, "b"),
            (SqlTokenType::And, "AND"),
            (SqlTokenType::Identifier_, "c"),
            (SqlTokenType::Between, "BETWEEN"),
            (SqlTokenType::NumberLiteral, "1"),
            (SqlTokenType::And, "AND"),
            (SqlTokenType::NumberLiteral, "2"),
            (SqlTokenType::Or, "OR"),
            (SqlTokenType::Identifier_, "d"),
            (SqlTokenType::In, "IN"),
            (SqlTokenType::LeftParen, "("),
            (SqlTokenType::NumberLiteral, "1"),
            (SqlTokenType::RightParen, ")"),
            (SqlTokenType::And, "AND"),
            (SqlTokenType::Identifier_, "e"),
            (SqlTokenType::Is, "IS"),
            (SqlTokenType::Null, "NULL"),
        ]
    );
}
