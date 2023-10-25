//! Bridge from `cst_format` toward `oak-formatter` on `RedTree`.
//!
//! RedTree formatting rules live in [`super::red_tree::TypeScriptRedTreeFormatter`].
//! `oak_formatter::Formatter` still lacks companion source text, so product callers use
//! `format_source` / `format_source_file` until `oak-formatter` carries source context.

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
