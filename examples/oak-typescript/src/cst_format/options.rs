//! CST format options and mapping to `oak-pretty-print` `PrinterConfig`.

use oak_pretty_print::{IndentStyle, Printer, PrinterConfig, document::Document};

/// Options for the CST format path.
#[derive(Debug, Clone)]
pub struct CstFormatOptions {
    /// Spaces per indent level (wired to `PrinterConfig` for future document rendering).
    pub indent_width: u8,
    /// Soft wrap width for future `Document` layout.
    pub line_width: usize,
}

impl Default for CstFormatOptions {
    fn default() -> Self {
        Self { indent_width: 4, line_width: 144 }
    }
}

impl CstFormatOptions {
    /// Map product-facing options to `oak-pretty-print` printer configuration.
    pub fn printer_config(&self) -> PrinterConfig {
        PrinterConfig::new().with_indent_style(IndentStyle::Spaces(self.indent_width)).with_max_width(self.line_width)
    }

    /// Apply printer finalization (trailing whitespace trim and final newline policy).
    pub fn finalize_output(&self, source: &str, body: String) -> String {
        let mut config = self.printer_config();
        config.insert_final_newline = source.ends_with('\n');
        let doc = Document::text(body);
        Printer::new(config).print(&doc)
    }
}
