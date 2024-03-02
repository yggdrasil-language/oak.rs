#![doc = include_str!("readme.md")]
#![deny(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;

/// Language configuration module.
pub mod language;
/// Lexer module.
pub mod lexer;

/// Parser module.
pub mod parser;

/// CST token-gap source formatter.
pub mod formatter;
/// AST pretty printer (`source` → parse → AST → text).
pub mod printer;

pub use crate::{
    ast::ValkyrieRoot,
    builder::ValkyrieBuilder,
    language::ValkyrieLanguage,
    lexer::{ValkyrieLexer, token_type::ValkyrieTokenType},
    parser::{ValkyrieParser, element_type::ValkyrieElementType},
};
pub use oak_core::{ElementType, TokenType};
