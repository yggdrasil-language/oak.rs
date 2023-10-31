//! Lexer-backed guards for preserving verbatim CST snippets.

use oak_core::{Lexer, ParseSession, SourceText};

use crate::{
    language::TypeScriptLanguage,
    lexer::{TypeScriptLexer, token_type::TypeScriptTokenType},
};

/// Keep snippet text when comments, decorators, or ASI-sensitive layout must survive.
pub fn should_preserve_verbatim(snippet: &str) -> bool {
    snippet_contains_comment(snippet) || snippet_has_decorator(snippet) || snippet_needs_asi_preservation(snippet)
}

fn snippet_has_decorator(snippet: &str) -> bool {
    let text = SourceText::new(snippet);
    let language = TypeScriptLanguage::default();
    let lexer = TypeScriptLexer::new(&language);
    let mut cache = ParseSession::default();
    let output = lexer.lex(&text, &[], &mut cache);
    match output.result {
        Ok(tokens) => tokens.iter().any(|token| matches!(token.kind, TypeScriptTokenType::At | TypeScriptTokenType::Decorator)),
        Err(_) => snippet.trim_start().starts_with('@'),
    }
}

fn snippet_contains_comment(snippet: &str) -> bool {
    let text = SourceText::new(snippet);
    let language = TypeScriptLanguage::default();
    let lexer = TypeScriptLexer::new(&language);
    let mut cache = ParseSession::default();
    let output = lexer.lex(&text, &[], &mut cache);
    match output.result {
        Ok(tokens) => tokens.iter().any(|token| matches!(token.kind, TypeScriptTokenType::LineComment | TypeScriptTokenType::BlockComment)),
        Err(_) => snippet.contains("//") || snippet.contains("/*"),
    }
}

/// Preserve snippets where a line break may change ASI grouping if rewritten on one line.
fn snippet_needs_asi_preservation(snippet: &str) -> bool {
    let mut lines = snippet.lines();
    lines.next();
    for line in lines {
        let trimmed = line.trim_start();
        if trimmed.is_empty() {
            continue;
        }
        let first = trimmed.as_bytes()[0];
        if matches!(first, b'+' | b'-' | b'(' | b'[' | b'.' | b'/') {
            return true;
        }
    }
    false
}
