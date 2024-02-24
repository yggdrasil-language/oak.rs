//! Printer 错误类型。

/// AST print 失败原因。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrintError {
    /// Oak 解析失败。
    Parse(String),
    /// 当前 printer 尚未覆盖的 AST 节点。
    Unsupported {
        /// 上下文描述（节点种类或路径）。
        context: String,
    },
}

impl std::fmt::Display for PrintError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Parse(message) => write!(f, "oak parse failed: {message}"),
            Self::Unsupported { context } => write!(f, "oak print unsupported: {context}"),
        }
    }
}

impl std::error::Error for PrintError {}
