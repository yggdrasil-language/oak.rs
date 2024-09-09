#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]

/// Abstract syntax tree for PlantUML class and sequence diagrams.
pub mod ast;
/// Builder that lowers the PlantUML green tree into the typed AST.
pub mod builder;
/// PlantUML language configuration.
pub mod language;
/// Line-aware lexer for the PlantUML subset.
pub mod lexer;
/// Parser that groups PlantUML statements into green-tree elements.
pub mod parser;

pub use crate::{
    ast::PlantUmlRoot,
    builder::PlantUmlBuilder,
    language::PlantUmlLanguage,
    lexer::{PlantUmlLexer, token_type::PlantUmlTokenType},
    parser::{PlantUmlParser, element_type::PlantUmlElementType},
};
