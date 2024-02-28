#![doc = include_str!("readme.md")]
#![deny(missing_docs)]
#![allow(missing_copy_implementations)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// The Markdown language implementation for Oaks.
pub mod language;

/// Abstract Syntax Tree for Markdown.
pub mod ast;
/// Builder for constructing Markdown syntax trees.
pub mod builder;

/// Syntax kinds for Markdown.
/// Lexer for tokenizing Markdown source.
pub mod lexer;
/// Language Server Protocol support for Markdown.
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;
/// MCP module.
#[cfg(feature = "mcp")]
pub mod mcp;

/// Parser for building Markdown syntax trees.
pub mod parser;

pub use crate::{ast::MarkdownRoot, builder::MarkdownBuilder, language::MarkdownLanguage, lexer::MarkdownLexer, parser::MarkdownParser};

#[cfg(feature = "oak-highlight")]
pub use crate::lsp::highlighter::MarkdownHighlighter;

// #[cfg(feature = "mcp")]
// pub use crate::mcp::serve_markdown_mcp;
pub use lexer::token_type::MarkdownTokenType;
pub use parser::element_type::MarkdownElementType;
