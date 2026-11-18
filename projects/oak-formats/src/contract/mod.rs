//! 前端无关的格式化基础设施（平台契约 + 语言插件实现）。
//!
//! - **本模块**：`FormatOptions` / `FormatError`、
//!   `Document` 布局引擎、`syntax` CST、`SourceMap`、`SourceFormatter` / `Printer` 与注册表。
//! - **语言插件**：在前端 crate 中按不透明 `language_id` 注册实现。

mod config;
mod document;
mod error;
mod options;
mod printer;
mod source;
mod source_map;
mod syntax;

pub use config::{FormatConfigLoader, FormatConfigResolution};
pub use document::Document;
pub use error::FormatError;
pub use options::FormatOptions;
pub use printer::{PrintStyle, Printer, PrinterProvider, PrinterRegistry};
pub use source::{SourceFormatter, SourceFormatterProvider, SourceFormatterRegistry};
pub use source_map::{FormattedOutput, SourceMap, SourceMapEntry};
pub use syntax::{ByteRange, SyntaxElement, SyntaxNode, SyntaxRoot, SyntaxToken, TriviaKind};
