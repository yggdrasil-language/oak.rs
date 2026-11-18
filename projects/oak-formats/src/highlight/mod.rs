//! 前端无关语法高亮基础设施（文本层，归 Oak）。
//!
//! - **本模块**：`HighlightKind` / `HighlightSpan` / `Highlighter` trait、
//!   `HighlighterKind`（Lexical / Semantic）、`HighlighterRegistry`、kind 映射工具与 HTML `hl-*`。
//! - **语言插件（前端 crate）**：同一语言可实现多个 `Highlighter`（词法快速、语义高质量）。

mod highlighter;
mod html;
mod kind;
mod span;

pub use highlighter::{
    AnalysisContext, HighlightRequest, Highlighter, HighlighterKind, HighlighterProvider, HighlighterRegistry, KindClassifier, SemanticKindMap,
    SyntaxHighlighter,
};
pub use html::{escape_html, render_spans_html};
pub use kind::HighlightKind;
pub use span::{HighlightSpan, merge_spans};
