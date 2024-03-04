#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST / typed view.
pub mod ast;
/// Builder module.
pub mod builder;
/// Language configuration.
pub mod language;
/// Lexer module.
pub mod lexer;
/// Parser module (GreenTree stub).
pub mod parser;
/// Typed recursive-descent syntax (compile contract).
pub mod syntax;

pub use crate::{
    ast::{Action, Axiom, BinOp, Connection, Formula, Island, Item, MetisRoot, Module, Param, Relation, Rewrites, Stmt, Theorem, TypeExpr, UnaryOp},
    builder::MetisBuilder,
    language::MetisLanguage,
    lexer::MetisLexer,
    parser::MetisParser,
    syntax::parse_module,
};
pub use lexer::token_type::MetisTokenType;
pub use parser::element_type::MetisElementType;

/// Foundation lex helper.
pub fn lex_stub(source: &str) -> Result<Vec<MetisTokenType>, String> {
    let tokens = lexer::lex_tokens(source)?;
    Ok(tokens.into_iter().map(|(k, _)| k).collect())
}
