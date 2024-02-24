//! Printer 选项（AST → 文本，非 CST formatter）。

/// 输出风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PrintStyle {
    /// 紧凑输出（最小空白）。
    #[default]
    Compact,
    /// 缩进多行。
    Indented,
}

/// Printer 选项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrintOptions {
    /// 每级缩进空格数。
    pub indent_width: usize,
}

impl Default for PrintOptions {
    fn default() -> Self {
        Self { indent_width: 4 }
    }
}
