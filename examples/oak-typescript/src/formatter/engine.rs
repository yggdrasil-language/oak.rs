use super::{FormatError, FormatOptions};

/// Returns `true` when the tree-aware path failed for a reason that must not fall back to
/// transitional AST output (invalid source, diagnostics, span integrity).
fn is_hard_format_error(message: &str) -> bool {
    message.contains("diagnostics")
        || message.contains("parse failed")
        || message.contains("overlapping CST spans")
}

/// Format TypeScript/JavaScript source text.
///
/// Uses a tree-aware path first, then a transitional AST output path for supported gaps.
/// Invalid or incomplete source must return [`Err`] without silent rewrite.
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let cst = options.cst_options();
    match crate::cst_format::format_source(source, &cst) {
        Ok(out) => Ok(out),
        Err(err) if is_hard_format_error(&err) => Err(FormatError::new(err)),
        Err(_) => crate::print::format_source(source, &crate::print::FormatOptions::default())
            .map_err(FormatError::new),
    }
}

#[cfg(test)]
mod smoke {
    use super::*;

    #[test]
    fn formats_const_spacing_and_idempotent() {
        let out = format_source("const  x=1", &FormatOptions::default()).expect("format");
        assert_eq!(out, "const x = 1");
        let again = format_source(&out, &FormatOptions::default()).expect("twice");
        assert_eq!(out, again);
    }
}
