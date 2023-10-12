#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![feature(portable_simd)]
#![warn(missing_docs)]
#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module for HTML nodes.
pub mod ast;
/// Builder module for constructing HTML trees.
pub mod builder;
/// Kind module defining HTML syntax types.
/// Language module for HTML configuration.
pub mod language;
/// Lexer module for HTML tokenization.
pub mod lexer;
/// LSP module for HTML language service features.
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;
/// MCP module.
#[cfg(feature = "mcp")]
pub mod mcp;

/// Parser module for HTML syntax analysis.
pub mod parser;
/// CSS selector subset queries over parsed HTML AST.
pub mod query;

pub use crate::{ast::HtmlDocument, builder::HtmlBuilder, language::HtmlLanguage, lexer::HtmlLexer, parser::HtmlParser};

/// Re-export of the HTML highlighter.
#[cfg(feature = "oak-highlight")]
pub use crate::lsp::highlighter::HtmlHighlighter;

/// Re-export of the HTML language service.
#[cfg(feature = "lsp")]
pub use crate::lsp::HtmlLanguageService;
pub use lexer::token_type::HtmlTokenType;
pub use parser::element_type::HtmlElementType;
