#![doc = include_str!("../readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// Typed AST for SMILES.
pub mod ast;
/// Builder turning the green tree into the typed AST.
pub mod builder;
/// Language definition for SMILES.
pub mod language;
/// Lexer for SMILES.
pub mod lexer;
/// Parser producing the green tree.
pub mod parser;

pub use crate::{
    ast::SmilesRoot, builder::SmilesBuilder, language::SmilesLanguage, lexer::SmilesLexer, parser::SmilesParser,
};

/// Parse a SMILES or reaction-SMILES source into its typed AST.
///
/// A syntax error is returned as the `Err` half of the result; the diagnostics
/// that led there are dropped by this convenience entry point. Use
/// [`SmilesBuilder`] directly when the diagnostics are needed.
pub fn parse_smiles(source: &str) -> Result<SmilesRoot, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};

    let builder = SmilesBuilder;
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::<SmilesLanguage>::default();
    builder.build(&text, &[], &mut cache).result.map_err(|error| format!("{error}"))
}
