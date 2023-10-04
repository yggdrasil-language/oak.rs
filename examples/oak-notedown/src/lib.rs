#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![warn(missing_docs)]

pub mod ast;
pub mod builder;

pub mod language;
pub mod lexer;
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;
/// MCP module.
#[cfg(feature = "mcp")]
pub mod mcp;

/// Parser functionality for Notedown.
pub mod parser;

pub use crate::{
    ast::NoteDocument as NoteRoot,
    builder::NoteBuilder,
    language::NotedownLanguage as NoteLanguage,
    lexer::{NotedownLexer as NoteLexer, token_type::NoteTokenType},
    parser::NoteParser,
};

#[cfg(feature = "lsp")]
pub use crate::lsp::highlighter::{HighlightKind, Highlighter, NoteHighlighter};

#[cfg(feature = "lsp")]
pub use crate::lsp::NoteLanguageService;

#[cfg(feature = "mcp")]
pub use crate::mcp::serve_note_mcp;
pub use parser::element_type::NoteElementType;

fn build_output(source: &str) -> oak_core::builder::BuildOutput<NoteLanguage> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};
    let config = NoteLanguage::default();
    let builder = NoteBuilder::new(&config);
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::default();
    builder.build(&text, &[], &mut cache)
}

/// Collect unified diagnostics from a Notedown parse without rendering to stderr.
#[cfg(feature = "diagnostic")]
pub fn diagnostic_set_from_notedown(source: &str) -> oak_core::diagnostic::DiagnosticSet {
    oak_core::diagnostic::diagnostic_set_from_output(&build_output(source))
}
