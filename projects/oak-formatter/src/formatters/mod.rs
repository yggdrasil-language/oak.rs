use core::range::Range;

use crate::{config::GapConstraint, errors::FormatResult};

/// A source edit produced by a formatter gap decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    /// The source gap replaced by this edit.
    pub span: Range<usize>,
    /// Replacement whitespace or trivia layout.
    pub text: String,
}

/// A token boundary and its source gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenGap<'a> {
    /// The preceding token text.
    pub left: &'a str,
    /// The source bytes between the two tokens.
    pub source: &'a str,
    /// The following token text.
    pub right: &'a str,
    /// The source range of the gap.
    pub span: Range<usize>,
}

/// Convert non-overlapping gap decisions into source edits.
pub fn edits_for_gaps<'a, I>(gaps: I) -> FormatResult<Vec<TextEdit>>
where
    I: IntoIterator<Item = (TokenGap<'a>, GapConstraint)>,
{
    let mut edits = Vec::new();
    let mut previous_end = 0;
    for (gap, constraint) in gaps {
        if gap.span.start < previous_end || gap.span.end < gap.span.start {
            return Err(oak_core::OakError::format_error("formatter produced overlapping token gaps"));
        }
        let text = match constraint {
            GapConstraint::NoSpace => String::new(),
            GapConstraint::RequiredSpace | GapConstraint::OptionalSpace => " ".to_owned(),
            GapConstraint::Preserve => gap.source.to_owned(),
            GapConstraint::HardLine => "\n".to_owned(),
        };
        if text != gap.source {
            edits.push(TextEdit { span: gap.span.clone(), text });
        }
        previous_end = gap.span.end;
    }
    Ok(edits)
}

/// Apply validated, source-relative edits from right to left.
pub fn apply_edits(source: &str, edits: &[TextEdit]) -> FormatResult<String> {
    let mut ordered = edits.to_vec();
    ordered.sort_by_key(|edit| (edit.span.start, edit.span.end));
    let mut previous_end = 0;
    for edit in &ordered {
        if edit.span.start < previous_end || edit.span.end > source.len() {
            return Err(oak_core::OakError::format_error("formatter edit range is invalid or overlapping"));
        }
        previous_end = edit.span.end;
    }
    let mut output = source.to_owned();
    for edit in ordered.into_iter().rev() {
        output.replace_range(edit.span.start..edit.span.end, &edit.text);
    }
    Ok(output)
}

/// Choose a conservative whitespace constraint for two lexical tokens.
pub fn conservative_constraint(left: &str, gap: &str, right: &str) -> GapConstraint {
    if left.starts_with("//") || left.starts_with("/*") || right.starts_with("//") || right.starts_with("/*") || gap.contains("//") || gap.contains("/*") || gap.contains('\n') {
        return GapConstraint::Preserve;
    }
    if needs_space(left, right) { GapConstraint::RequiredSpace } else { GapConstraint::NoSpace }
}

fn needs_space(left: &str, right: &str) -> bool {
    if right.is_empty() {
        return false;
    }
    if (left == "<" && right == "/") || (left == "/" && right == ">") {
        return false;
    }
    // Delimiters and member access bind directly to their neighbours.
    if matches!(right, ")" | "]" | "," | ";" | "." | "?.")
        || matches!(left, "(" | "[" | "." | "?.")
        || (left == "{" && right == "}")
        || (left == "?" && right == ":")
        || (left == "/" && right.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic()))
        || (left == "!" && right != "=")
        || left == "~"
        || matches!(left, "++" | "--")
        || matches!(right, "++" | "--")
    {
        return false;
    }
    // Optional properties and parameters use `name?: Type`.
    if left == "?" && right == ":" {
        return false;
    }
    if left == "?" {
        return true;
    }
    if right == ":" {
        return false;
    }
    // Control-flow keywords conventionally separate from their condition.
    if matches!(left, "if" | "for" | "while" | "switch" | "catch" | "with") && right == "(" {
        return true;
    }
    if matches!(right, "as" | "instanceof" | "in") || matches!(left, "as" | "instanceof" | "in") {
        return true;
    }
    if matches!(left, "return" | "throw" | "new" | "typeof" | "void" | "delete" | "await" | "yield")
        && matches!(right, "[" | "{" | "(")
    {
        return true;
    }
    let left_word = left.chars().last().is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$');
    let right_word = right.chars().next().is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$');
    let left_quote = left.ends_with(['\'', '"', '`']);
    let right_quote = right.starts_with(['\'', '"', '`']);
    let left_operator = left.ends_with(['=', '+', '-', '*', '/', '%', ':', '&', '|']) || matches!(left, "??" | "??=" | "&&" | "&&=" | "||" | "||=" | "=>" | "<=" | ">=" | "==" | "===");
    let right_operator = right.starts_with(['=', '+', '-', '*', '/', '%', ':', '&', '|']) || matches!(right, "??" | "??=" | "&&" | "&&=" | "||" | "||=" | "=>" | "<=" | ">=" | "==" | "===" | "!==");
    let brace_boundary = left.ends_with('{') || right.starts_with('}') || right == "{" || (left_word && right.starts_with('{')) || (left.ends_with('}') && right_word);

    (left_word && (right_word || right_quote)) || (left_quote && right_word) || left.ends_with(',') || left.ends_with(';') || left.ends_with(':') || left_operator || right_operator || (left_word && right == "<") || (left == ">" && right_word) || brace_boundary
}

#[cfg(test)]
mod tests {
    use super::conservative_constraint;
    use crate::GapConstraint;

    fn space(left: &str, right: &str) -> bool {
        matches!(conservative_constraint(left, "", right), GapConstraint::RequiredSpace)
    }

    #[test]
    fn keeps_typescript_optional_markers_tight() {
        assert!(!space("name", "?"));
        assert!(!space("?", ":"));
        assert!(space(":", "string"));
    }

    #[test]
    fn keeps_unary_and_update_operators_tight() {
        assert!(!space("!", "ready"));
        assert!(!space("++", "index"));
        assert!(!space("index", "++"));
    }

    #[test]
    fn separates_control_flow_conditions() {
        assert!(space("if", "("));
        assert!(space("for", "("));
        assert!(!space("call", "("));
    }

    #[test]
    fn separates_comparison_operators_without_touching_jsx_openers() {
        assert!(space("left", "<"));
        assert!(space(">", "right"));
        assert!(!space("<", "div"));
    }

    #[test]
    fn separates_type_assertions_and_keyword_expressions() {
        assert!(space(")", "as"));
        assert!(space("as", "string"));
        assert!(space("return", "["));
        assert!(!space("{", "}"));
    }
}
