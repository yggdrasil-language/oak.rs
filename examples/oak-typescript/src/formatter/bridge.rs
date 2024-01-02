//! TypeScript adapter for the Oak token-gap formatter.

use super::{options::FormatOptions, source::format_source};

/// Formats TypeScript and JavaScript source through the CST token-gap path.
#[derive(Debug, Clone, Copy, Default)]
pub struct TypeScriptCstFormatter;

impl TypeScriptCstFormatter {
    /// Format one source buffer without constructing a pretty-print document.
    pub fn format_source(&self, source: &str, options: &FormatOptions) -> Result<String, oak_core::OakError> {
        format_source(source, options)
    }
}
