//! Transitional TypeScript **AST print** (`source` → parse → AST → text).
//!
//! This is **not** a CST-faithful formatter and does not use `oak-pretty-print` /
//! `oak-formatter`. See `tests/format_print_contract.rs` and the VMZ capability matrix.
//! Returns [`Err`] when parse fails or the AST contains unprintable nodes.

mod expr;
mod jsx;
mod stmt;

use oak_core::{Builder, ParseSession, SourceText};

use crate::{TypeScriptBuilder, TypeScriptLanguage};

use expr::print_expression;

/// Minimal formatter options for the Oak print path.
#[derive(Debug, Clone, Default)]
pub struct FormatOptions {}

/// Format TypeScript/JavaScript source via Oak parse + AST print.
pub fn format_source(source: &str, _options: &FormatOptions) -> Result<String, String> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &text, &[], &mut cache);

    if let Err(err) = &built.result {
        return Err(format!("oak parse failed: {err:?}"));
    }
    if !built.diagnostics.is_empty() {
        return Err(format!("oak diagnostics: {:?}", built.diagnostics));
    }

    let root = built.result.ok().ok_or_else(|| "oak build returned no root".to_string())?;
    let mut out = String::new();
    for (index, stmt) in root.statements.iter().enumerate() {
        if index > 0 {
            out.push('\n');
        }
        out.push_str(&stmt::print_statement(stmt).ok_or_else(|| {
            format!("oak print unsupported for statement: {stmt:?}")
        })?);
    }

    if source.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_const_spacing() {
        let out = format_source("const  x=1", &FormatOptions::default()).expect("format");
        assert_eq!(out, "const x = 1");
    }

    #[test]
    fn format_is_idempotent() {
        let once = format_source("const x = 1", &FormatOptions::default()).expect("once");
        let twice = format_source(&once, &FormatOptions::default()).expect("twice");
        assert_eq!(once, twice);
    }

    #[test]
    fn ast_print_drops_leading_line_comment() {
        let out = format_source("// keep\nconst x = 1", &FormatOptions::default()).expect("format");
        assert_eq!(out, "const x = 1");
        assert!(!out.contains("keep"));
    }
}
