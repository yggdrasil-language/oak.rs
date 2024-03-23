#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;
/// Kind definition module.
/// Language configuration module.
pub mod language;
/// Lexer module.
pub mod lexer;
/// LSP module.
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "mcp"))]
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;
/// MCP module.
#[cfg(feature = "mcp")]
pub mod mcp;

/// Parser module.
pub mod parser;

pub use crate::{builder::YamlBuilder, language::YamlLanguage, lexer::YamlLexer, parser::YamlParser};

/// Highlighter implementation.
#[cfg(feature = "oak-highlight")]
pub use crate::lsp::highlighter::YamlHighlighter;

/// Parses a YAML string.
pub fn parse(yaml: &str) -> Result<crate::ast::YamlRoot, String> {
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};
    let language = YamlLanguage::new();
    let builder = YamlBuilder::new(&language);
    let source = SourceText::new(yaml.to_string());
    let mut cache = ParseSession::default();
    let result = builder.build(&source, &[], &mut cache);
    result.result.map_err(|e| format!("{:?}", e))
}

/// Deserializes a YAML string into a Rust type.
#[cfg(feature = "serde")]
pub use crate::language::from_str;

/// Serializes a Rust type into a YAML string.
#[cfg(feature = "serde")]
pub use crate::language::to_string;

/// LSP implementation.
#[cfg(feature = "lsp")]
pub use crate::lsp::YamlLanguageService;
#[cfg(feature = "oak-pretty-print")]
pub use crate::lsp::formatter::YamlFormatter;

#[cfg(feature = "mcp")]
pub use crate::mcp::serve_yaml_mcp;
pub use lexer::token_type::YamlTokenType;
pub use parser::element_type::YamlElementType;
