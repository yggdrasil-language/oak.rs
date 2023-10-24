//! Bridge from `cst_format` toward `oak-formatter` on `RedTree`.
//!
//! `oak_formatter::Formatter` formats a `RedTree` without source text. TypeScript CST
//! formatting needs both the red-green tree and the original `source`, so the product
//! entry remains `format_source` / `format_source_file` until `oak-formatter` carries
//! source context or pretty-print rules consume trivia-aware documents end to end.

use oak_core::RedNode;

use crate::language::TypeScriptLanguage;

use super::{CstFormatOptions, format_source_file};

/// TypeScript CST formatter adapter for RedTree + source pairs.
///
/// A future `oak_formatter::Formatter<TypeScriptLanguage>` impl still needs source text
/// alongside the tree. Until then, product callers use `format_source` or this adapter.
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeScriptCstFormatter;

impl TypeScriptCstFormatter {
    /// Format one parsed `SourceFile` node using CST span gaps and statement rules.
    pub fn format_source_file(
        source: &str,
        file: &RedNode<'_, TypeScriptLanguage>,
        options: &CstFormatOptions,
    ) -> Result<String, String> {
        format_source_file(source, file, options)
    }
}
