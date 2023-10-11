#![doc = include_str!("readme.md")]
#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]

/// AST module.
pub mod ast;
/// Builder module.
pub mod builder;

// pub mod formatter;
// pub mod highlighter;
/// Type definitions module.
/// Language configuration module.
pub mod language;
/// Lexer module.
pub mod lexer;
/// LSP module.
#[cfg(any(feature = "lsp", feature = "oak-highlight", feature = "oak-pretty-print"))]
pub mod lsp;

// pub mod mcp;
/// Parser module.
pub mod parser;
/// XPath subset queries over parsed XML AST.
pub mod query;

pub use crate::{
    ast::{XmlNode, XmlValue},
    language::XmlLanguage,
    lexer::{XmlLexer, token_type::XmlTokenType},
    parser::XmlParser,
};

// #[cfg(feature = "oak-highlight")]
// pub use crate::lsp::highlighter::XmlHighlighter;

// /// LSP implementation.
#[cfg(feature = "lsp")]
pub use crate::lsp::XmlLanguageService;
// #[cfg(feature = "lsp")]
// pub use crate::lsp::formatter::XmlFormatter;

#[cfg(feature = "serde")]
pub use crate::language::serde::{from_value, to_value};

/// Serializes a value to an XML string.
#[cfg(feature = "serde")]
pub fn to_string<T: ::serde::Serialize>(value: &T) -> Result<String, String> {
    let xml_value = to_value(value).map_err(String::from)?;
    Ok(xml_value.to_string())
}

/// Deserializes a value from an XML string.
#[cfg(feature = "serde")]
pub fn from_str<T: ::serde::de::DeserializeOwned>(s: &str) -> Result<T, String> {
    let xml_value = parse(s)?;
    from_value(xml_value).map_err(String::from)
}

fn build_output(xml: &str) -> oak_core::builder::BuildOutput<XmlLanguage> {
    use crate::builder::XmlBuilder;
    use oak_core::{Builder, parser::session::ParseSession, source::SourceText};
    let builder = XmlBuilder::new();
    let source = SourceText::new(xml.to_string());
    let mut cache = ParseSession::default();
    builder.build(&source, &[], &mut cache)
}

/// Parses an XML string into an `XmlValue`.
pub fn parse(xml: &str) -> Result<XmlValue, String> {
    let output = build_output(xml);
    output
        .result
        .map(|root| root.value)
        .map_err(|error| format!("Parse failed: {:?}, diagnostics: {:?}", error, output.diagnostics))
}

/// Collect unified diagnostics from an XML parse without rendering to stderr.
#[cfg(feature = "diagnostic")]
pub fn diagnostic_set_from_xml(xml: &str) -> oak_core::diagnostic::DiagnosticSet {
    oak_core::diagnostic::diagnostic_set_from_output(&build_output(xml))
}

pub use parser::element_type::XmlElementType;
