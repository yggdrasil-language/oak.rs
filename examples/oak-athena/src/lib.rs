#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![warn(missing_docs)]

pub mod builder;
pub mod language;
pub mod lexer;
pub mod parser;

pub use crate::{
    builder::AthenaBuilder,
    language::AthenaLanguage,
    lexer::AthenaLexer,
    parser::AthenaParser,
};
pub use lexer::token_type::AthenaTokenType;
pub use oak_core::{ElementType, TokenType};
pub use parser::element_type::AthenaElementType;
