#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;

/// Type definitions module.
/// Language configuration module.
pub mod language;
/// Lexer module.
pub mod lexer;
/// LSP module.
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;
/// MCP module.
#[cfg(feature = "mcp")]
pub mod mcp;

/// Parser module.
pub mod parser;

pub use crate::{ast::HlslRoot, builder::HlslBuilder, language::HlslLanguage, lexer::HlslLexer, parser::HlslParser};

/// Parses HLSL source into the typed AST.
///
/// Syntax errors are returned as `Err` with the diagnostic message. Callers
/// that need the full diagnostics should use [`HlslBuilder`] directly.
pub fn parse_hlsl(source: &str) -> Result<HlslRoot, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};

    let language = HlslLanguage::default();
    let builder = HlslBuilder::new(&language);
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::<HlslLanguage>::default();
    builder.build(&text, &[], &mut cache).result.map_err(|error| error.to_string())
}

/// Highlighter implementation.
#[cfg(feature = "oak-highlight")]
pub use crate::lsp::highlighter::HlslHighlighter;

/// LSP implementation.
#[cfg(feature = "lsp")]
pub use crate::lsp::HlslLanguageService;
pub use lexer::token_type::HlslTokenType;
pub use parser::element_type::HlslElementType;
