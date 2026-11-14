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
/// Parser module.
pub mod parser;

pub use crate::{ast::GlslRoot, builder::GlslBuilder, language::GlslLanguage, lexer::GlslLexer, parser::GlslParser};

/// Parses GLSL source into the typed AST.
///
/// Syntax errors are returned as `Err` with the diagnostic message. Callers
/// that need the full diagnostics should use [`GlslBuilder`] directly.
pub fn parse_glsl(source: &str) -> Result<GlslRoot, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};

    let language = GlslLanguage::default();
    let builder = GlslBuilder::new(&language);
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::<GlslLanguage>::default();
    builder.build(&text, &[], &mut cache).result.map_err(|error| error.to_string())
}

pub use lexer::token_type::GlslTokenType;
pub use parser::element_type::GlslElementType;
