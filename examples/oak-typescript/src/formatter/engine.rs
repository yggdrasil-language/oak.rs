use super::{FormatError, FormatOptions};

/// Format TypeScript/JavaScript source text.
///
/// Formatting is exclusively CST based. AST printing is not a formatter fallback.
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    if options.type_erasure {
        return Err(oak_core::OakError::format_error("type erasure is not part of the CST formatter contract"));
    }
    super::source::format_source(source, options)
}
