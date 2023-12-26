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


    pub(crate) fn printer_config(&self) -> oak_pretty_print::PrinterConfig {
        oak_pretty_print::PrinterConfig::new()
            .with_indent_style(oak_pretty_print::IndentStyle::Spaces(self.indent_width))
            .with_max_width(self.line_width)
    }

    pub(crate) fn finalize_output(&self, source: &str, body: String) -> String {
        let mut config = self.printer_config();
        config.insert_final_newline = source.ends_with('\n');
        let doc = oak_pretty_print::document::Document::text(body);
        oak_pretty_print::Printer::new(config).print(&doc)
    }

}
