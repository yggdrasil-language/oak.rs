//! VON **源码**格式化（CST 路径；与 [`crate::printer::print_von`] AST printer 分离）。
//!
//! `.von` 正规格式化委托 `oak_von::formatter`（Oak token-gap）。

use crate::formatter::{FormatError, FormatOptions, FormattedOutput};

/// 格式化 `.von` 源码（保留 `#` 注释 trivia）。
pub(crate) fn format_von_source(source: &str, options: &FormatOptions) -> Result<FormattedOutput, FormatError> {
    format_von_with_oak(source, options)
}

fn format_von_with_oak(source: &str, options: &FormatOptions) -> Result<FormattedOutput, FormatError> {
    let oak_options = oak_von::formatter::FormatOptions { indent_width: options.indent_width.min(255) as u8, line_width: options.max_width };
    let text = oak_von::formatter::format_source(source, &oak_options)
        .map_err(|error| FormatError::Parse { path: None, message: error.to_string() })?;
    let mut out = text;
    if options.ensure_trailing_newline && !out.is_empty() && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(FormattedOutput::text_only(out))
}
