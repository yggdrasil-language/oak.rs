#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yggdrasil-language/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/yggdrasil-language/oaks/refs/heads/dev/documents/logo.svg")]

/// The Oak language configuration for VOS.
pub mod language;
/// The Oak lexer for VOS.
pub mod lexer;
/// The Oak parser for VOS.
pub mod parser;

pub use crate::{
    language::VosLanguage,
    lexer::{VosLexer, VosToken, VosTokenType},
    parser::{VosElementType, VosParser},
};

/// A source-preserving VOS parse result.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VosRoot {
    /// The exact source passed to the Oak frontend.
    pub source: String,
}

/// Parses VOS through Oak's lexer and parser pipeline.
pub fn parse(source: &str) -> Result<VosRoot, String> {
    use oak_core::{Parser, SourceText, parser::session::ParseSession};

    let language = VosLanguage::default();
    let parser = VosParser::new(&language);
    let source_text = SourceText::new(source.to_owned());
    let mut cache = ParseSession::<VosLanguage>::default();
    let result = parser.parse(&source_text, &[], &mut cache);
    result.result.map(|_| VosRoot { source: source.to_owned() }).map_err(|error| format!("{error:?}"))
}
