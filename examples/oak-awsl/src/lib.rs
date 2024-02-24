#![doc = include_str!("readme.md")]

#![feature(new_range_api)]

#![warn(missing_docs)]



mod frontend;



/// AWSL component ABI (`[property]` / `[event]` / `[memoize]`).

pub mod abi;

/// AWSL AST nodes.

pub mod ast;

/// Component naming helpers.

pub mod component;

/// Parse errors.

pub mod error;

/// CST token-gap source formatter (markup-aware lossless scan).

pub mod formatter;

/// HTML void-element helpers.

pub mod html;

/// AWSL lexer.

pub mod lexer;

/// AWSL parser.

pub mod parser;



pub use abi::{

    AbiDerived, AbiEffect, AbiEvent, AbiExtractResult, AbiIssue, AbiIssueKind, AbiMemo, AbiParam, AbiProperty, AbiReference, AbiSeverity,

    AbiState, AbiSymbolKind, ComponentAbi, ComponentAbiIndex, TemplateBinding, TemplateBindingKind, abi_declaration_span, classify_abi_cursor,

    collect_abi_references, collect_template_bindings, extract_component_abi_from_script, extract_component_abi_from_vx,

    find_template_binding_at, is_snake_case, normalize_event_name, normalize_prop_name, refine_binding_kind,

};

pub use ast::{

    AwslAttribute, AwslAttributeValue, AwslDirective, AwslDirectiveKind, AwslElement, AwslImport, AwslRoot, AwslTemplateNode, AwslTextPart,

};

pub use component::{awsl_stem_from_component_tag, resolve_widget_name, validate_component_contract, widget_name_from_stem};

pub use error::AwslParseError;

pub use html::{HTML_VOID_ELEMENTS, is_html_void_element};

pub use lexer::Lexer;

pub use parser::AwslParser;



/// Parse an AWSL source file into [`AwslRoot`].

pub fn parse_root(source: &str) -> Result<AwslRoot, AwslParseError> {

    AwslParser::parse_root(source)

}



/// Parse an AWSL source file with strict-mode options.

pub fn parse_root_with_options(source: &str, strict_mode: bool) -> Result<AwslRoot, AwslParseError> {

    AwslParser::parse_root_with_options(source, strict_mode)

}


