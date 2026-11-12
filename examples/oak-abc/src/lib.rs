#![doc = include_str!("../readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// Typed AST for ABC notation.
pub mod ast;
/// Builder turning the green tree into the typed AST.
pub mod builder;
/// Language definition for ABC notation.
pub mod language;
/// Line-oriented lexer.
pub mod lexer;
/// Parser producing the green tree.
pub mod parser;

pub use crate::{
    ast::AbcRoot, builder::AbcBuilder, language::AbcLanguage, lexer::AbcLexer, parser::AbcParser,
};

/// Parse an ABC source into its typed AST.
///
/// A syntax error is returned as the `Err` half of the result; the diagnostics
/// that led there are dropped by this convenience entry point. Use
/// [`AbcBuilder`] directly when the diagnostics are needed.
pub fn parse_abc(source: &str) -> Result<AbcRoot, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};

    let builder = AbcBuilder;
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::<AbcLanguage>::default();
    builder.build(&text, &[], &mut cache).result.map_err(|error| format!("{error}"))
}
