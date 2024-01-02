#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]

mod printer;

/// AST module for TypeScript.
pub mod ast;
/// Builder module for TypeScript.
pub mod builder;
/// Source formatting through the CST token-gap adapter.
pub mod formatter;
/// Language definition for TypeScript.
pub mod language;
/// Lexer for TypeScript.
pub mod lexer;
/// Parser for TypeScript.
pub mod parser;

pub use crate::{
    ast::TypeScriptRoot,
    builder::TypeScriptBuilder,
    formatter::{FormatError, FormatOptions, TypeScriptCstFormatter, format_source},
    language::TypeScriptLanguage,
    lexer::{TypeScriptLexer, token_type::TypeScriptTokenType},
    parser::{TypeScriptParser, element_type::TypeScriptElementType},
};

#[cfg(feature = "lsp")]
pub use crate::lsp::{TypeScriptLanguageService, formatter::TypeScriptFormatter, highlighter::TypeScriptHighlighter};

#[cfg(feature = "mcp")]
pub use crate::mcp::serve_typescript_mcp;
