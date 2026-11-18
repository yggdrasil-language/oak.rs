//! Formatting options shared by the `Document` engine, printers and source formatters.

/// 格式化 / printer 布局选项。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormatOptions {
    /// 每级缩进空格数。
    pub indent_width: usize,
    /// pretty-print 目标行宽（字符数）；`0` 表示始终折行。
    pub max_width: usize,
    /// 非空文件是否以换行结尾。
    pub ensure_trailing_newline: bool,
    /// LSP：制表符宽度（`insert_spaces` 为 true 时映射到 `indent_width`）。
    pub tab_size: u32,
    /// LSP：用空格代替制表符。
    pub insert_spaces: bool,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self { indent_width: 4, max_width: 80, ensure_trailing_newline: true, tab_size: 4, insert_spaces: true }
    }
}

impl FormatOptions {
    /// 从 LSP 风格选项构造。
    pub fn from_lsp(tab_size: u32, insert_spaces: bool) -> Self {
        let indent = if insert_spaces { tab_size as usize } else { tab_size as usize };
        Self { indent_width: indent.max(1), tab_size, insert_spaces, ..Self::default() }
    }
}
