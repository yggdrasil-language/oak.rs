use super::{FormatError, FormatOptions};

/// Format AWSL (`.awsl`) source text.
///
/// Formatting is exclusively CST token-gap based. AST printing is not a formatter fallback.
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    if source.is_empty() {
        return Ok(String::new());
    }
    super::source::format_source(source, options)
}
