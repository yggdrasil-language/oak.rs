//! AWSL markup token-gap adapter (lossless scan until Oak parser lands).

use oak_formatter::{GapConstraint, TokenGap, apply_edits, conservative_constraint, edits_for_gaps};

use super::FormatOptions;

struct ScannedToken {
    span: core::range::Range<usize>,
}

pub(crate) fn format_source(source: &str, options: &FormatOptions) -> Result<String, oak_core::OakError> {
    let tokens = scan_significant_tokens(source)?;
    validate_delimiters(source, &tokens)?;

    let mut gaps = Vec::new();
    for pair in tokens.windows(2) {
        let left = &pair[0];
        let right = &pair[1];
        let span = core::range::Range { start: left.span.end, end: right.span.start };
        let gap = source.get(span.clone()).ok_or_else(|| oak_core::OakError::format_error("token span is outside source"))?;
        let left_text = source.get(left.span.clone()).ok_or_else(|| oak_core::OakError::format_error("left token span is outside source"))?;
        let right_text = source.get(right.span.clone()).ok_or_else(|| oak_core::OakError::format_error("right token span is outside source"))?;
        let constraint = awsl_gap_constraint(left_text, gap, right_text);
        gaps.push((TokenGap { left: left_text, source: gap, right: right_text, span }, constraint));
    }

    let edits = edits_for_gaps(gaps)?;
    let formatted = apply_edits(source, &edits)?;
    let _ = options;
    Ok(formatted)
}

fn scan_significant_tokens(source: &str) -> Result<Vec<ScannedToken>, oak_core::OakError> {
    let mut tokens = Vec::new();
    let mut index = 0usize;
    while index < source.len() {
        if let Some(skip) = skip_whitespace(source, index) {
            index = skip;
            continue;
        }
        if let Some((token_end, _)) = scan_html_comment(source, index) {
            tokens.push(ScannedToken { span: core::range::Range { start: index, end: token_end } });
            index = token_end;
            continue;
        }
        if let Some((token_end, _)) = scan_quoted(source, index) {
            tokens.push(ScannedToken { span: core::range::Range { start: index, end: token_end } });
            index = token_end;
            continue;
        }
        let ch = source[index..].chars().next().ok_or_else(|| oak_core::OakError::format_error("invalid utf-8 in source"))?;
        if is_punct(ch) {
            tokens.push(ScannedToken { span: core::range::Range { start: index, end: index + ch.len_utf8() } });
            index += ch.len_utf8();
            continue;
        }
        let start = index;
        while index < source.len() {
            if skip_whitespace(source, index).is_some() {
                break;
            }
            if scan_html_comment(source, index).is_some() || scan_quoted(source, index).is_some() {
                break;
            }
            let next = source[index..].chars().next().unwrap();
            if is_punct(next) {
                break;
            }
            index += next.len_utf8();
        }
        if index == start {
            return Err(oak_core::OakError::format_error("failed to advance AWSL scan"));
        }
        tokens.push(ScannedToken { span: core::range::Range { start, end: index } });
    }
    Ok(tokens)
}

fn skip_whitespace(source: &str, index: usize) -> Option<usize> {
    let mut pos = index;
    while pos < source.len() {
        match source[pos..].chars().next() {
            Some(' ' | '\t' | '\n' | '\r') => pos += 1,
            _ => break,
        }
    }
    if pos > index { Some(pos) } else { None }
}

fn scan_html_comment(source: &str, index: usize) -> Option<(usize, ())> {
    if !source[index..].starts_with("<!--") {
        return None;
    }
    let close = source[index..].find("-->").map(|offset| index + offset + 3)?;
    Some((close, ()))
}

fn scan_quoted(source: &str, index: usize) -> Option<(usize, ())> {
    let quote = source[index..].chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut pos = index + quote.len_utf8();
    while pos < source.len() {
        let ch = source[pos..].chars().next().unwrap();
        if ch == '\\' {
            pos += ch.len_utf8();
            if pos < source.len() {
                pos += source[pos..].chars().next().unwrap().len_utf8();
            }
            continue;
        }
        pos += ch.len_utf8();
        if ch == quote {
            return Some((pos, ()));
        }
    }
    None
}

fn is_punct(ch: char) -> bool {
    matches!(ch, '<' | '>' | '/' | '=' | '{' | '}' | '@' | ':' | ',' | '(' | ')' | '[' | ']')
}

fn should_preserve_gap(left: &str, gap: &str, right: &str) -> bool {
    gap.contains('\n')
        || left.starts_with("<!--")
        || right.starts_with("<!--")
        || gap.contains("<!--")
        || left.starts_with('#')
        || right.starts_with('#')
}

fn awsl_gap_constraint(left: &str, gap: &str, right: &str) -> GapConstraint {
    if should_preserve_gap(left, gap, right) {
        return GapConstraint::Preserve;
    }

    let mut constraint = conservative_constraint(left, gap, right);

    if right == "=" || left == "=" {
        constraint = GapConstraint::NoSpace;
    }
    if left == "<" || right == ">" {
        constraint = GapConstraint::NoSpace;
    }
    if left == "/" && right == ">" {
        constraint = GapConstraint::NoSpace;
    }
    if left == ">" && right == "<" {
        constraint = GapConstraint::Preserve;
    }
    if left == ">" && !right.starts_with('<') {
        constraint = GapConstraint::RequiredSpace;
    }
    if right == "{" || left == "}" {
        constraint = GapConstraint::NoSpace;
    }
    if left == "{" {
        constraint = GapConstraint::NoSpace;
    }

    constraint
}

fn validate_delimiters(source: &str, tokens: &[ScannedToken]) -> Result<(), oak_core::OakError> {
    let mut stack = Vec::new();
    for token in tokens {
        let text = source.get(token.span.clone()).ok_or_else(|| oak_core::OakError::format_error("token span is outside source"))?;
        if text.starts_with("<!--") || text.starts_with('"') || text.starts_with('\'') {
            continue;
        }
        match text {
            "<" => stack.push('<'),
            ">" => {
                if stack.pop() != Some('<') {
                    return delimiter_error(source, token.span.start);
                }
            }
            "{" => stack.push('{'),
            "}" => {
                if stack.pop() != Some('{') {
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
