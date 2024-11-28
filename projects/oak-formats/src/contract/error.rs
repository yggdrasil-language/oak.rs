//! 格式化错误类型。

use std::path::PathBuf;

/// 格式化 / printer 错误。
#[derive(Debug)]
pub enum FormatError {
    /// 不支持的语言或扩展名。
    Unsupported {
        /// 语言 id 或扩展名。
        language: String,
        /// 相关路径（若有）。
        path: Option<PathBuf>,
    },
    /// 读取/写入失败。
    Io {
        /// 相关路径。
        path: PathBuf,
        /// 底层 IO 错误。
        source: std::io::Error,
    },
    /// 源码或文档解析失败。
    Parse {
        /// 相关路径（若有）。
        path: Option<PathBuf>,
        /// 错误说明。
        message: String,
    },
    /// 文档类型与 printer 不匹配。
    WrongDocument {
        /// 期望的文档类型说明。
        expected: String,
    },
}

impl FormatError {
    /// 不支持的扩展名（路径已知）。
    pub fn unsupported_extension(path: PathBuf) -> Self {
        let language = path.extension().and_then(|e| e.to_str()).unwrap_or("").to_string();
        Self::Unsupported { language, path: Some(path) }
    }

    /// 不支持的语言 id。
    pub fn unsupported_language(language: impl Into<String>) -> Self {
        Self::Unsupported { language: language.into(), path: None }
    }
}

impl std::fmt::Display for FormatError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unsupported { language, path } => match path {
                Some(path) => write!(f, "不支持的语言/扩展名 `{language}`：{}", path.display()),
                None => write!(f, "不支持的语言：{language}"),
            },
            Self::Io { path, source } => write!(f, "读写失败 {}: {}", path.display(), source),
            Self::Parse { path, message } => match path {
                Some(path) => write!(f, "解析失败 {}: {message}", path.display()),
                None => write!(f, "解析失败: {message}"),
            },
            Self::WrongDocument { expected } => write!(f, "文档类型不匹配，期望 {expected}"),
        }
    }
}

impl std::error::Error for FormatError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}
