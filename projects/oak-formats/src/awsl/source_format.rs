//! AWSL **源码**格式化（Oak token-gap；与 AST printer 分离）。

use crate::formatter::{FormatError, FormatOptions, FormattedOutput};

pub(crate) fn format_awsl(source: &str, options: &FormatOptions) -> Result<FormattedOutput, FormatError> {
    let oak_options = oak_awsl::formatter::FormatOptions { indent_width: options.indent_width.min(255) as u8, line_width: options.max_width };
    let text = oak_awsl::formatter::format_source(source, &oak_options)
        .map_err(|error| FormatError::Parse { path: None, message: error.to_string() })?;
    let mut out = text;
    if options.ensure_trailing_newline && !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(FormattedOutput::text_only(out))
}
