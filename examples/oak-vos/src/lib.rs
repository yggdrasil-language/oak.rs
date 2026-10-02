#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/yggdrasil-language/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/yggdrasil-language/oaks/refs/heads/dev/documents/logo.svg")]

/// The Oak language configuration for VOS.
pub mod language;
/// The initial declaration AST emitted by Oak Builder.
pub mod ast;
/// The Oak Builder for VOS declarations.
pub mod builder;
/// The Oak lexer for VOS.
pub mod lexer;
/// The Oak parser for VOS.
pub mod parser;

pub use crate::{
    ast::{VosDeclaration, VosDeclarationKind, VosRoot},
    builder::VosBuilder,
    language::VosLanguage,
    lexer::{VosLexer, VosToken, VosTokenType},
    parser::{VosElementType, VosParser},
};

/// Builds a structural VOS CST through Oak's lexer and parser pipeline.
///
/// This entry rejects lexical errors and malformed delimiters. It does not
/// establish full VOS grammar validity or produce a resolved VOS contract.
pub fn parse(source: &str) -> Result<VosRoot, String> {
    use oak_core::{Builder, SourceText, parser::session::ParseSession};

    let language = VosLanguage::default();
    let builder = VosBuilder::new(&language);
    let source_text = SourceText::new(source.to_owned());
    let mut cache = ParseSession::<VosLanguage>::default();
    let result = builder.build(&source_text, &[], &mut cache);
    if let Some(error) = result.diagnostics.first() {
        return Err(format!("{error:?}"));
    }
    result.result.map_err(|error| format!("{error:?}"))
}
