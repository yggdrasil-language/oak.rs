#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

extern crate alloc;

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;
/// CST token-gap source formatter.
pub mod formatter;
/// Language configuration module.
pub mod language;
/// Lexer module.
pub mod lexer;

pub mod parser;

pub use crate::{
    ast::VonValue as VonValueNode,
    builder::VonBuilder,
    language::{VonLanguage, VonValue},
    lexer::VonLexer,
    parser::VonParser,
};

#[cfg(feature = "serde")]
pub use crate::language::{from_str, to_string};

/// Parses a Von string into a `VonValueNode`.
pub fn parse(von: &str) -> Result<VonValueNode, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};
    let language = VonLanguage::default();
    let builder = VonBuilder::new(&language);
    let source = SourceText::new(von.to_string());
    let mut cache = ParseSession::default();
    let result = builder.build(&source, &[], &mut cache);
    result.result.map(|root| root.value).map_err(|e| format!("{:?}", e))
}

pub use lexer::token_type::VonTokenType;
pub use parser::element_type::VonElementType;
