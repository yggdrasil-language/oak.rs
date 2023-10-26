#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]

mod cst_format;
mod print;

/// AST module for TypeScript.
pub mod ast;
/// Builder module for TypeScript.
pub mod builder;
/// Source formatting (`format_source`, `FormatOptions`).
pub mod formatter;

/// Language definition for TypeScript.
pub mod language;
/// Lexer for TypeScript.
pub mod lexer;

/// Parser for TypeScript.
pub mod parser;

// Re-exports
pub use crate::{
    ast::TypeScriptRoot,
    builder::TypeScriptBuilder,
    formatter::{FormatError, FormatOptions, format_source},
    language::TypeScriptLanguage,
    lexer::{TypeScriptLexer, token_type::TypeScriptTokenType},
    parser::{TypeScriptParser, element_type::TypeScriptElementType},
};

#[cfg(feature = "lsp")]
pub use crate::lsp::{TypeScriptLanguageService, formatter::TypeScriptFormatter, highlighter::TypeScriptHighlighter};

#[cfg(feature = "mcp")]
pub use crate::mcp::serve_typescript_mcp;
