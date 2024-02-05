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
    for (index, pair) in significant.windows(2).enumerate() {
        let left = &pair[0];
        let right = &pair[1];
        let span = Range { start: left.span.end, end: right.span.start };
        let gap = source.get(span.clone()).ok_or_else(|| oak_core::OakError::format_error("lexer token span is outside source"))?;
        let left_text = source.get(left.span.clone()).ok_or_else(|| oak_core::OakError::format_error("left token span is outside source"))?;
        let right_text = source.get(right.span.clone()).ok_or_else(|| oak_core::OakError::format_error("right token span is outside source"))?;
        let mut constraint = conservative_constraint(left_text, gap, right_text);
        if left_text == ">" && is_jsx_attribute_gap(&significant, index, source) {
            constraint = oak_formatter::GapConstraint::NoSpace;
        }
        if right_text.starts_with(['\'', '"', '`']) && left_text == "=" && is_jsx_attribute_gap(&significant, index, source) {
            constraint = oak_formatter::GapConstraint::NoSpace;
        }
        if right_text == "=" && is_jsx_attribute_gap(&significant, index, source) {
            constraint = oak_formatter::GapConstraint::NoSpace;
        }
        if left_text == "=" && is_jsx_attribute_gap(&significant, index, source) {
            constraint = oak_formatter::GapConstraint::NoSpace;
        }
        if right_text.starts_with(['\'', '"', '`']) && significant.get(index.wrapping_sub(1)).and_then(|token| source.get(token.span.clone())) == Some("=") && is_jsx_attribute_gap(&significant, index, source) {
            constraint = oak_formatter::GapConstraint::NoSpace;
        }
        // `?` is ambiguous in a token pair. In `name?: Type` it binds to the
        // property name, while in `condition ? value : fallback` it is a
        // ternary operator. Use the next CST token to disambiguate it.
        if right_text == "?" {
            let next = significant.get(index + 2).and_then(|token| source.get(token.span.clone()));
            constraint = if next == Some(":") { oak_formatter::GapConstraint::NoSpace } else { oak_formatter::GapConstraint::RequiredSpace };
        }
        if right_text == ":" {
            let mut ternary = false;
            if left_text != "?" {
                let mut nesting = match left_text {
                    ")" | "]" | "}" => 1,
                    _ => 0,
                };
                for token in significant[..index].iter().rev().take(64) {
                    match source.get(token.span.clone()) {
                        Some("?") if token.kind == TypeScriptTokenType::Question && nesting == 0 => {
                            ternary = true;
                            break;
                        }
                        Some("}") => break,
                        Some(")" | "]") => nesting += 1,
                        Some("(" | "[" | "{") if nesting > 0 => nesting -= 1,
                        Some("(" | "[" | "{") => break,
                        Some(";" | ",") if nesting == 0 => break,
                        _ => {}
                    }
                }
            }
            constraint = if ternary { oak_formatter::GapConstraint::RequiredSpace } else { oak_formatter::GapConstraint::NoSpace };
        }
        if right_text == "<" || right_text == ">" {
            let next = significant.get(index + 2).and_then(|token| source.get(token.span.clone()));
            if right_text == "<" && next == Some("/") {
                constraint = oak_formatter::GapConstraint::NoSpace;
            }
            else if right_text == "<" && looks_like_type_arguments(&significant, index + 1, source) {
                constraint = oak_formatter::GapConstraint::NoSpace;
            }
            else if right_text == ">" && is_type_argument_close(&significant, index + 1, source) {
                constraint = oak_formatter::GapConstraint::NoSpace;
            }
            else if right_text == ">" && significant.get(index.wrapping_sub(1)).and_then(|token| source.get(token.span.clone())).is_some_and(|text| text == "<" || text == "/") {
                constraint = oak_formatter::GapConstraint::NoSpace;
            }
            else if left_text.chars().last().is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | ')' | ']')) {
                constraint = oak_formatter::GapConstraint::RequiredSpace;
            }
        }
        if left_text == "<" || left_text == ">" {
            let generic_angle = (left_text == "<" && looks_like_type_arguments(&significant, index, source))
                || (left_text == ">" && is_type_argument_close(&significant, index, source));
            let comparison = !generic_angle && significant.get(index.wrapping_sub(1)).and_then(|token| source.get(token.span.clone())).is_some_and(|text| text.chars().last().is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '$' | ')' | ']')));
            if !right_text.is_empty() && comparison && right_text != "/" {
                constraint = oak_formatter::GapConstraint::RequiredSpace;
            }
            if generic_angle && (left_text == "<" || matches!(right_text, "{" | "(" | ")" | "," | ";" | ">" | ">>")) {
                constraint = oak_formatter::GapConstraint::NoSpace;
            }
        }
        if (left_text == "async" && right_text == "(")
            || (matches!(left_text, "const" | "let" | "var") && right_text == "[")
            || (left_text == "]" && right_text == "of") {
            constraint = oak_formatter::GapConstraint::RequiredSpace;
        }
        if gap.contains('\n') || left_text.starts_with("//") || left_text.starts_with("/*") || right_text.starts_with("//") || right_text.starts_with("/*")
            || (source.starts_with("#!") && left.span.start < source.find('\n').unwrap_or(source.len())) {
            constraint = oak_formatter::GapConstraint::Preserve;
        }
        gaps.push((TokenGap { left: left_text, source: gap, right: right_text, span }, constraint));
    }
    let edits = edits_for_gaps(gaps)?;
    let formatted = apply_edits(source, &edits)?;
    let _ = options;
    Ok(formatted)
}

fn is_jsx_attribute_gap(significant: &[&oak_core::Token<TypeScriptTokenType>], index: usize, source: &str) -> bool {
    let mut nesting = 0usize;
    for (token_index, token) in significant[..=index].iter().enumerate().rev() {
        let text = match source.get(token.span.clone()) {
            Some(text) => text,
            None => return false,
        };
        match text {
            ">" => nesting += 1,
            "<" if nesting == 0 => return true,
            "<" if nesting > 0 => {
                nesting -= 1;
                if nesting == 0 {
                    let previous = token_index.checked_sub(1).and_then(|previous| significant.get(previous)).and_then(|token| source.get(token.span.clone()));
                    return matches!(previous, Some("=" | "return" | "=>" | "(" | "["));
                }
            }
            ";" | "{" | "}" if nesting == 0 => return false,
            _ => {}
        }
    }
    false
}

fn looks_like_type_arguments(significant: &[&oak_core::Token<TypeScriptTokenType>], index: usize, source: &str) -> bool {
    let mut depth = 0usize;
    let mut delimiters = 0usize;
    for token in significant.iter().skip(index + 1) {
        let text = match source.get(token.span.clone()) {
            Some(text) => text,
            None => return false,
        };
        match text {
            "{" | "(" | "[" => delimiters += 1,
            "}" | ")" | "]" if delimiters > 0 => delimiters -= 1,
            "<" => depth += 1,
            ">" if depth == 0 && delimiters == 0 => {
                let after = significant.iter().skip_while(|candidate| candidate.span.start <= token.span.start).find_map(|candidate| source.get(candidate.span.clone()));
                return after.is_none_or(|next| matches!(next, "=" | "," | ";" | ")" | "]" | "}" | "." | "(" | "{" | ">" | "|" | "&" | "=>" | "?" | ":"));
            }
            ">" => depth -= 1,
            ">>" if depth <= 2 && delimiters == 0 => {
                let after = significant.iter().skip_while(|candidate| candidate.span.start <= token.span.start).find_map(|candidate| source.get(candidate.span.clone()));
                return after.is_none_or(|next| matches!(next, "=" | "," | ";" | ")" | "]" | "}" | "." | "(" | "{" | "|" | "&" | "=>" | "?" | ":"));
            }
            ">>" => depth -= 2,
            ";" | "=" | "?" | ":" if depth == 0 && delimiters == 0 => return false,
            _ => {}
        }
    }
    false
}

fn is_type_argument_close(significant: &[&oak_core::Token<TypeScriptTokenType>], close_index: usize, source: &str) -> bool {
    let mut depth = 0usize;
    for index in (0..close_index).rev() {
        let text = match source.get(significant[index].span.clone()) {
            Some(text) => text,
            None => return false,
        };
        match text {
            ">" => depth += 1,
            "<" if depth == 0 => return looks_like_type_arguments(significant, index, source),
            "<" => depth -= 1,
            ";" | "=" | "?" | ":" if depth == 0 => return false,
            _ => {}
        }
    }
    false
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

#[cfg(test)]
mod tests {
    use super::format_source;
    use crate::formatter::FormatOptions;

    #[test]
    fn keeps_generic_types_tight_through_nested_closers() {
        let source = "async function load(value: Record<string, unknown>): Promise<Array<number>> { return new Set<string>(); } const defaults: Record<string, string> = {};";
        assert_eq!(format_source(source, &FormatOptions::default()).unwrap(), source);
    }

    #[test]
    fn keeps_function_return_annotation_tight_after_optional_parameters() {
        let source = "function parse(fallback?: string): Record<string, unknown> { return {}; }";
        assert_eq!(format_source(source, &FormatOptions::default()).unwrap(), source);
    }

    #[test]
    fn preserves_authoring_boundaries() {
        for source in ["#!/usr/bin/env node\nconst value = 1;", "const task = async (value) => value;", "for (const [key, value] of entries) { consume(key, value); }", "type Rows = Array<{ id: string }>;", "const value = ready\n    ? load()\n    : fallback;"] {
            assert_eq!(format_source(source, &FormatOptions::default()).unwrap(), source);
        }
    }
}
