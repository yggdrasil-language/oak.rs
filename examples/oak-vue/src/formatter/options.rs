use oak_pretty_print::{IndentStyle, Printer, PrinterConfig, document::Document};

/// Product-facing options for [`super::format_source`].
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FormatOptions {
    /// Spaces per indent level.
    pub indent_width: u8,
    /// Soft wrap width for formatted output.
    pub line_width: usize,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent_width: 4, line_width: 144 }
    }
}

impl FormatOptions {
    fn printer_config(&self) -> PrinterConfig {
        PrinterConfig::new().with_indent_style(IndentStyle::Spaces(self.indent_width)).with_max_width(self.line_width)
    }

    /// Apply printer finalization (trailing whitespace trim and final newline policy).
    pub(crate) fn finalize_output(&self, source: &str, body: String) -> String {
        let mut config = self.printer_config();
        config.insert_final_newline = source.ends_with('\n');
        let doc = Document::text(body);
        Printer::new(config).print(&doc)
    }
}
