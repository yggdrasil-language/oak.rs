//! Printer 选项（`Document` 渲染配置）。

use oak_pretty_print::document::printer::{IndentStyle, LineEnding, PrinterConfig};

/// 输出风格（`PrintOptions` 字段）。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintStyle {
    /// 尽量单行（`max_width = usize::MAX`）。
    #[default]
    Compact,
    /// 按 `max_width` 折行缩进。
    Indented,
}

/// Printer 选项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintOptions {
    /// 输出风格。
    pub style: PrintStyle,
    /// 每级缩进空格数。
    pub indent_width: usize,
    /// `style == Indented` 时的目标行宽。
    pub max_width: usize,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self { style: PrintStyle::Compact, indent_width: 4, max_width: 80 }
    }
}

impl PrintOptions {
    /// 紧凑单行输出。
    pub fn compact() -> Self {
        Self::default()
    }

    /// 缩进折行输出（默认 `max_width = 80`）。
    pub fn indented() -> Self {
        Self { style: PrintStyle::Indented, indent_width: 4, max_width: 80 }
    }

    /// 将 printer 选项映射为 `oak_pretty_print` 渲染配置。
    pub fn printer_config(&self) -> PrinterConfig {
        let indent_width = self.indent_width.clamp(1, 8);
        let mut config = PrinterConfig::default();
        config.indent_style = IndentStyle::Spaces(indent_width as u8);
        config.indent_text = " ".repeat(indent_width).into();
        config.indent_size = indent_width;
        config.insert_final_newline = false;
        config.line_ending = LineEnding::Unix;
        config.max_width = match self.style {
            PrintStyle::Compact => usize::MAX,
            PrintStyle::Indented => self.max_width,
        };
        config
    }
}
