#![doc = include_str!("readme.md")]
#![warn(missing_docs)]

pub mod ast;
pub mod builder;
pub mod language;
pub mod lexer;
pub mod parser;

pub use crate::{builder::MapleBuilder, language::MapleLanguage, lexer::MapleLexer, parser::MapleParser};
pub use lexer::token_type::MapleTokenType;
pub use parser::element_type::MapleElementType;
