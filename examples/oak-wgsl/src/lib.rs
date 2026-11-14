#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;

/// Language definition.
pub mod language;
/// Lexer module.
pub mod lexer;
/// LSP module.
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;
/// Parser module.
pub mod parser;

pub use crate::{ast::WgslRoot, builder::WgslBuilder, language::WgslLanguage, lexer::WgslLexer, parser::WgslParser};

/// Parses WGSL source into the typed AST.
///
/// Syntax errors are returned as `Err` with the diagnostic message. Callers
/// that need the full diagnostics should use [`WgslBuilder`] directly.
pub fn parse_wgsl(source: &str) -> Result<WgslRoot, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};

    let language = WgslLanguage::new();
    let builder = WgslBuilder::new(&language);
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::<WgslLanguage>::default();
    builder.build(&text, &[], &mut cache).result.map_err(|error| error.to_string())
}

/// LSP implementation.
#[cfg(feature = "lsp")]
pub use crate::lsp::WgslLanguageService;
pub use lexer::token_type::WgslTokenType;
pub use parser::element_type::WgslElementType;
