#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;
/// Language module.
pub mod language;
/// Lexer module.
pub mod lexer;

/// Source formatting (`format_source`, `FormatOptions`).
pub mod formatter;
/// Parser module.
pub mod parser;

pub use ast::*;
pub use builder::VueBuilder;
pub use formatter::{FormatError, FormatOptions, format_source};
pub use language::VueLanguage;
pub use lexer::{VueLexer, token_type::VueTokenType};
pub use parser::{VueParser, element_type::VueElementType};

// pub fn parse<'a>(source: &'a SourceText, cache: &'a mut oak_core::parser::ParseSession<crate::lexer::token_type::VueLanguage>) -> ParseOutput<'a, crate::lexer::token_type::VueLanguage> {
// let language = crate::lexer::token_type::VueLanguage::default();
// let lexer = VueLexer::new(&language);
// let parser = VueParser::new(&language);
// oak_core::parser::parse(&parser, &lexer, source, &[], cache)
// }
