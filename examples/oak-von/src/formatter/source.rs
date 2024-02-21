//! VON CST token-gap adapter.

use core::range::Range;

use oak_core::{Lexer, ParseSession, SourceText};
use oak_formatter::{GapConstraint, TokenGap, apply_edits, conservative_constraint, edits_for_gaps};

use super::FormatOptions;
use crate::{
    language::VonLanguage,
    lexer::{VonLexer, token_type::VonTokenType},
};

pub(crate) fn format_source(source: &str, options: &FormatOptions) -> Result<String, oak_core::OakError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = VonLanguage::default();
    let lexer = VonLexer::new(&language);
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
    for (index, pair) in significant.windows(2).enumerate() {
        let left = &pair[0];
        let right = &pair[1];
        let span = Range { start: left.span.end, end: right.span.start };
        let gap = source.get(span.clone()).ok_or_else(|| oak_core::OakError::format_error("lexer token span is outside source"))?;
        let left_text = source.get(left.span.clone()).ok_or_else(|| oak_core::OakError::format_error("left token span is outside source"))?;
        let right_text = source.get(right.span.clone()).ok_or_else(|| oak_core::OakError::format_error("right token span is outside source"))?;
        let constraint = von_gap_constraint(left_text, gap, right_text, left.kind, right.kind, index, &significant, source);
        gaps.push((TokenGap { left: left_text, source: gap, right: right_text, span }, constraint));
    }

    let edits = edits_for_gaps(gaps)?;
    let formatted = apply_edits(source, &edits)?;
    let _ = options;
    Ok(formatted)
}

fn is_layout(kind: VonTokenType) -> bool {
    matches!(kind, VonTokenType::Whitespace | VonTokenType::Newline)
}

fn should_preserve_gap(left: &str, gap: &str, right: &str, left_kind: VonTokenType, right_kind: VonTokenType) -> bool {
    gap.contains('\n')
        || left.starts_with('#')
        || right.starts_with('#')
        || gap.trim_start().starts_with('#')
        || matches!(left_kind, VonTokenType::Comment)
        || matches!(right_kind, VonTokenType::Comment)
}

fn von_gap_constraint(
    left: &str,
    gap: &str,
    right: &str,
    left_kind: VonTokenType,
    right_kind: VonTokenType,
    _index: usize,
    _significant: &[&oak_core::Token<VonTokenType>],
    _source: &str,
) -> GapConstraint {
    if should_preserve_gap(left, gap, right, left_kind, right_kind) {
        return GapConstraint::Preserve;
    }

    let mut constraint = conservative_constraint(left, gap, right);

    if right == ":" {
        constraint = GapConstraint::NoSpace;
    }
    if left == ":" {
        constraint = GapConstraint::RequiredSpace;
    }
    if left == "," {
        constraint = GapConstraint::RequiredSpace;
    }
    if matches!(left, "{" | "[") {
        constraint = GapConstraint::RequiredSpace;
    }
    if matches!(right, "}" | "]") {
        constraint = GapConstraint::RequiredSpace;
    }
    if left == "{" && right == "}" {
        constraint = GapConstraint::NoSpace;
    }
    if left == "[" && right == "]" {
        constraint = GapConstraint::NoSpace;
    }

    constraint
}

fn validate_delimiters(source: &str, tokens: &[&oak_core::Token<VonTokenType>]) -> Result<(), oak_core::OakError> {
    let mut stack = Vec::new();
    for token in tokens {
        if matches!(
            token.kind,
            VonTokenType::StringLiteral | VonTokenType::NumberLiteral | VonTokenType::Comment | VonTokenType::BoolLiteral | VonTokenType::NullLiteral
        ) {
            continue;
        }
        match token.kind {
            VonTokenType::LeftBrace => stack.push('{'),
            VonTokenType::LeftBracket => stack.push('['),
            VonTokenType::LeftParen => stack.push('('),
            VonTokenType::RightBrace => {
                if stack.pop() != Some('{') {
                    return delimiter_error(source, token.span.start);
                }
            }
            VonTokenType::RightBracket => {
                if stack.pop() != Some('[') {
                    return delimiter_error(source, token.span.start);
                }
            }
            VonTokenType::RightParen => {
                if stack.pop() != Some('(') {
                    return delimiter_error(source, token.span.start);
                }
            }
            _ => {}
        }
    }
    if stack.is_empty() {
        Ok(())
    }
    else {
        Err(oak_core::OakError::format_error(format!("formatter input has unbalanced delimiters: unclosed {stack:?}")))
    }
}

fn delimiter_error(source: &str, pos: usize) -> Result<(), oak_core::OakError> {
    let snippet = source.get(pos.saturating_sub(40)..pos.saturating_add(40)).unwrap_or("");
    Err(oak_core::OakError::format_error(format!("formatter input has unbalanced delimiters at {pos}: {snippet:?}")))
}
