//! TypeScript CST token-gap adapter.

use core::range::Range;
use oak_core::{Lexer, ParseSession, SourceText};
use oak_formatter::{TokenGap, apply_edits, conservative_constraint, edits_for_gaps};

use super::FormatOptions;
use crate::{
    language::TypeScriptLanguage,
    lexer::{TypeScriptLexer, token_type::TypeScriptTokenType},
};

pub(crate) fn format_source(source: &str, options: &FormatOptions) -> Result<String, oak_core::OakError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = TypeScriptLanguage::default();
    let lexer = TypeScriptLexer::new(&language);
    let mut session = ParseSession::default();
    let output = lexer.lex(&text, &[], &mut session);
    let tokens = match output.result {
        Ok(tokens) => tokens,
        Err(error) => return Err(error),
    };
    if let Some(error) = output.diagnostics.into_iter().next() {
        return Err(error);
    }
    let significant: Vec<_> = tokens.iter().filter(|token| !is_layout(token.kind)).collect();
    validate_delimiters(source, &significant)?;
    let mut gaps = Vec::new();
    for pair in significant.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let span = Range { start: left.span.end, end: right.span.start };
        let gap = source.get(span.clone()).ok_or_else(|| oak_core::OakError::format_error("lexer token span is outside source"))?;
        let left_text = source.get(left.span.clone()).ok_or_else(|| oak_core::OakError::format_error("left token span is outside source"))?;
        let right_text = source.get(right.span.clone()).ok_or_else(|| oak_core::OakError::format_error("right token span is outside source"))?;
        gaps.push((TokenGap { left: left_text, source: gap, right: right_text, span }, conservative_constraint(left_text, gap, right_text)));
    }
    let edits = edits_for_gaps(gaps)?;
    let formatted = apply_edits(source, &edits)?;
    let _ = options;
    Ok(formatted)
}

fn is_layout(kind: TypeScriptTokenType) -> bool {
    matches!(kind, TypeScriptTokenType::Whitespace | TypeScriptTokenType::Newline)
}

fn validate_delimiters(source: &str, tokens: &[&oak_core::Token<TypeScriptTokenType>]) -> Result<(), oak_core::OakError> {
    let mut stack = Vec::new();
    for token in tokens {
        let text = source.get(token.span.clone()).ok_or_else(|| oak_core::OakError::format_error("lexer token span is outside source"))?;
        match text {
            "{" | "(" | "[" => stack.push(text),
            "}" | ")" | "]" => {
                let expected = match text {
                    "}" => "{",
                    ")" => "(",
                    "]" => "[",
                    _ => unreachable!(),
                };
                if stack.pop() != Some(expected) {
                    return Err(oak_core::OakError::format_error("formatter input has unbalanced delimiters"));
                }
            }
            _ => {}
        }
    }
    if stack.is_empty() { Ok(()) } else { Err(oak_core::OakError::format_error("formatter input has unbalanced delimiters")) }
}
