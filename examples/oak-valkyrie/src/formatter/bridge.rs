//! Valkyrie adapter for the Oak token-gap formatter.

use super::{options::FormatOptions, source::format_source};

/// Formats Valkyrie source through the CST token-gap path.
#[derive(Debug, Clone, Copy, Default)]
pub struct ValkyrieCstFormatter;

impl ValkyrieCstFormatter {
    /// Format one source buffer without constructing a pretty-print document.
    pub fn format_source(&self, source: &str, options: &FormatOptions) -> Result<String, oak_core::OakError> {
        format_source(source, options)
    }
}
