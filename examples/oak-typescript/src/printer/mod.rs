//! Transitional TypeScript **AST print** (`source` → parse → AST → text).
//!
//! This is **not** a CST-faithful formatter and does not use `oak-pretty-print` /
//! Returns [`Err`] when parse fails or the AST contains unprintable nodes.
//! Product contract tests live in downstream `nifty-formatter` / `vmz-formatter`.

mod expr;
mod jsx;
mod stmt;

use oak_core::{Builder, ParseSession, SourceText};

use crate::{TypeScriptBuilder, TypeScriptLanguage};


/// Minimal formatter options for the Oak print path.
#[derive(Debug, Clone, Default)]
pub struct FormatOptions {
    /// Erase TypeScript-only syntax while printing.
    pub type_erasure: bool,
}

/// Format TypeScript/JavaScript source via Oak parse + AST print.
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String, String> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language).with_type_erasure(options.type_erasure);
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
        if options.type_erasure
            && matches!(stmt, crate::ast::Statement::ImportDeclaration(import) if import.is_type_only)
        {
            continue;
        }
        if !out.is_empty() {
            out.push('\n');
        }
        out.push_str(&stmt::print_statement(stmt).ok_or_else(|| format!("oak print unsupported for statement: {stmt:?}"))?);
    }

    if source.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }

    Ok(out)
}
