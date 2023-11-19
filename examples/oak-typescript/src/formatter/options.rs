use crate::cst_format::CstFormatOptions;

/// Product-facing options for [`super::format_source`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FormatOptions {
    /// Spaces per indent level.
    pub indent_width: u8,
    /// Soft wrap width for formatted output.
    pub line_width: usize,
    /// Erase TypeScript-only syntax and print a JavaScript module.
    pub type_erasure: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent_width: 4, line_width: 144, type_erasure: false }
    }
}

impl FormatOptions {
    /// Enable or disable TypeScript type erasure.
    pub fn with_type_erasure(mut self, enabled: bool) -> Self {
        self.type_erasure = enabled;
        self
    }

    pub(crate) fn cst_options(&self) -> CstFormatOptions {
        CstFormatOptions { indent_width: self.indent_width, line_width: self.line_width }
    }
}
