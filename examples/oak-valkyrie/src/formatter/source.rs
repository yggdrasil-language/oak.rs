//! Valkyrie CST token-gap adapter.

use core::range::Range;

use oak_core::{Lexer, ParseSession, SourceText};
use oak_formatter::{GapConstraint, TokenGap, apply_edits, conservative_constraint, edits_for_gaps};

use super::FormatOptions;
use crate::{
    language::ValkyrieLanguage,
    lexer::{ValkyrieLexer, token_type::ValkyrieTokenType},
};

pub(crate) fn format_source(source: &str, options: &FormatOptions) -> Result<String, oak_core::OakError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = ValkyrieLanguage::default();
    let lexer = ValkyrieLexer::new(&language);
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
        let constraint = valkyrie_gap_constraint(left_text, gap, right_text, left.kind, right.kind, index, &significant, source);
        gaps.push((TokenGap { left: left_text, source: gap, right: right_text, span }, constraint));
    }

    let edits = edits_for_gaps(gaps)?;
    let formatted = apply_edits(source, &edits)?;
    let _ = options;
    Ok(formatted)
}

fn is_layout(kind: ValkyrieTokenType) -> bool {
    matches!(kind, ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline)
}

fn should_preserve_gap(left: &str, gap: &str, right: &str) -> bool {
    gap.contains('\n')
        || left.starts_with('#')
        || right.starts_with('#')
        || left.starts_with("<#")
        || right.starts_with("<#")
        || gap.contains("<#")
        || gap.contains("#>")
        || gap.trim_start().starts_with('#')
}

fn valkyrie_gap_constraint(
    left: &str,
    gap: &str,
    right: &str,
    left_kind: ValkyrieTokenType,
    right_kind: ValkyrieTokenType,
    index: usize,
    significant: &[&oak_core::Token<ValkyrieTokenType>],
    source: &str,
) -> GapConstraint {
    if should_preserve_gap(left, gap, right) {
        return GapConstraint::Preserve;
    }

    let mut constraint = conservative_constraint(left, gap, right);

    if matches!(
        left,
        "micro"
            | "mezzo"
            | "macro"
            | "let"
            | "mut"
            | "return"
            | "raise"
            | "yield"
            | "if"
            | "while"
            | "loop"
            | "until"
            | "match"
            | "namespace"
            | "using"
            | "class"
            | "structure"
            | "widget"
            | "singleton"
            | "neural"
            | "trait"
            | "imply"
            | "unite"
            | "union"
            | "enums"
            | "flags"
            | "type"
            | "attribute"
            | "tests"
            | "break"
            | "continue"
            | "fallthrough"
            | "resume"
            | "where"
            | "as"
            | "is"
            | "pin"
            | "own"
            | "catch"
    ) && (right_kind == ValkyrieTokenType::Identifier || right == "(" || right == "{")
    {
        constraint = GapConstraint::RequiredSpace;
    }

    if left == ")" && right == "{" {
        constraint = GapConstraint::RequiredSpace;
    }

    if left == "}" && right_kind == ValkyrieTokenType::Identifier {
        constraint = GapConstraint::RequiredSpace;
    }

    if left == ";" && right_kind == ValkyrieTokenType::Identifier {
        constraint = GapConstraint::RequiredSpace;
    }

    if right == "<" && looks_like_generic_open(significant, index + 1, source) {
        constraint = GapConstraint::NoSpace;
    }
    if left == ">" && is_generic_close(significant, index, source) {
        if right == "{" {
            constraint = GapConstraint::RequiredSpace;
        }
        else if matches!(right, "(" | ")" | "," | ";" | ">" | "|" | "&" | "?" | ":") {
            constraint = GapConstraint::NoSpace;
        }
    }

    if matches!(left_kind, ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment)
        || matches!(right_kind, ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment)
    {
        constraint = GapConstraint::Preserve;
    }

    constraint
}

fn looks_like_generic_open(significant: &[&oak_core::Token<ValkyrieTokenType>], open_index: usize, source: &str) -> bool {
    let mut depth = 0usize;
    for token in significant.iter().skip(open_index + 1) {
        let text = match source.get(token.span.clone()) {
            Some(text) => text,
            None => return false,
        };
        match text {
            "<" => depth += 1,
            ">" if depth == 0 => return true,
            ">" => depth -= 1,
            ";" | "{" | "}" if depth == 0 => return false,
            _ => {}
        }
    }
    false
}

fn is_generic_close(significant: &[&oak_core::Token<ValkyrieTokenType>], close_index: usize, source: &str) -> bool {
    let mut depth = 0usize;
    for index in (0..close_index).rev() {
        let text = match source.get(significant[index].span.clone()) {
            Some(text) => text,
            None => return false,
        };
        match text {
            ">" => depth += 1,
            "<" if depth == 0 => return looks_like_generic_open(significant, index, source),
            "<" => depth -= 1,
            ";" | "=" | "?" | ":" if depth == 0 => return false,
            _ => {}
        }
    }
    false
}

fn validate_delimiters(source: &str, tokens: &[&oak_core::Token<ValkyrieTokenType>]) -> Result<(), oak_core::OakError> {
    let mut stack = Vec::new();
    for token in tokens {
        if matches!(
            token.kind,
            ValkyrieTokenType::StringLiteral
                | ValkyrieTokenType::CharLiteral
                | ValkyrieTokenType::LineComment
                | ValkyrieTokenType::BlockComment
        ) {
            continue;
        }
        match token.kind {
            ValkyrieTokenType::LeftBrace => stack.push('{'),
            ValkyrieTokenType::LeftParen => stack.push('('),
            ValkyrieTokenType::LeftBracket => stack.push('['),
            ValkyrieTokenType::LeftOffset => stack.push('\u{2045}'),
            ValkyrieTokenType::RightBrace => {
                if stack.pop() != Some('{') {
                    return delimiter_error(source, token.span.start);
                }
            }
            ValkyrieTokenType::RightParen => {
                if stack.pop() != Some('(') {
                    return delimiter_error(source, token.span.start);
                }
            }
            ValkyrieTokenType::RightBracket => {
                if stack.pop() != Some('[') {
                    return delimiter_error(source, token.span.start);
                }
            }
            ValkyrieTokenType::RightOffset => {
                if stack.pop() != Some('\u{2045}') {
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
