use crate::{
    ValkyrieLanguage,
    ast::*,
    builder::{ValkyrieBuilder, text, utils},
    lexer::token_type::ValkyrieTokenType,
    parser::{parse_string_segments, parse_string_segments::parse_string_segments},
};
use oak_core::{OakError, RedNode, RedTree, Source};

/// Counts the number of leading double quotes in a string.
fn count_leading_quotes(text: &str) -> u8 {
    let mut count = 0u8;
    for ch in text.chars() {
        if ch == '"' {
            count += 1;
        }
        else {
            break;
        }
    }
    count
}

/// Extracts the content of a string literal by removing leading and trailing quotes.
fn extract_content(text: &str, quote_count: u8) -> &str {
    let start = quote_count as usize;
    let end = text.len().saturating_sub(quote_count as usize);
    if start >= end {
        return "";
    }
    &text[start..end]
}

impl<'config> ValkyrieBuilder<'config> {
    pub(crate) fn build_literal<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut prefix: Option<Identifier> = None;
        let mut string_value: Option<String> = None;
        let mut string_span: Option<oak_core::Range<usize>> = None;

        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                match t.kind {
                    ValkyrieTokenType::Whitespace | ValkyrieTokenType::Newline | ValkyrieTokenType::LineComment | ValkyrieTokenType::BlockComment => continue,
                    ValkyrieTokenType::StringPrefix => {
                        let prefix_text = text(source, t.span);
                        prefix = Some(Identifier { name: prefix_text, span: t.span });
                    }
                    ValkyrieTokenType::StringLiteral => {
                        string_value = Some(text(source, t.span));
                        string_span = Some(t.span);
                    }
                    ValkyrieTokenType::IntegerLiteral => {
                        return Ok(TermExpression::IntegerLiteral { value: text(source, t.span), span });
                    }
                    ValkyrieTokenType::FloatLiteral => {
                        return Ok(TermExpression::FloatLiteral { value: text(source, t.span), span });
                    }
                    _ => {}
                }
            }
        }

        if let (Some(raw_text), Some(str_span)) = (string_value, string_span) {
            let quote_count = count_leading_quotes(&raw_text);
            let content = extract_content(&raw_text, quote_count);
            let is_raw = prefix.as_ref().map(|p| p.name == "r").unwrap_or(false);
            let content_start = str_span.start + quote_count as usize;
            let segments = parse_string_segments(content, content_start, is_raw);

            Ok(TermExpression::StringLiteral(StringLiteral { prefix, quote_count, segments, span }))
        }
        else {
            Err(source.syntax_error("Missing string literal value".to_string(), span.start))
        }
    }

    pub(crate) fn build_bool_literal<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let mut value = false;
        for child in node.children() {
            if let RedTree::Leaf(t) = child {
                if t.kind == ValkyrieTokenType::BoolLiteral {
                    let text_val = text(source, t.span);
                    value = text_val == "true";
                }
            }
        }
        Ok(TermExpression::Bool { value, span })
    }

    pub(crate) fn build_identifier_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let span = node.span();
        let identifier = utils::build_identifier(&node, source, span)?;
        Ok(TermExpression::NamePath(Box::new(NamePath { parts: vec![identifier.clone()], span: identifier.span })))
    }

    pub(crate) fn build_path_expr<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<TermExpression, OakError> {
        let path = utils::build_name_path(&node, source)?;
        Ok(TermExpression::NamePath(Box::new(path)))
    }

    pub(crate) fn build_name_path<S: Source + ?Sized>(&self, node: RedNode<ValkyrieLanguage>, source: &S) -> Result<NamePath, OakError> {
        utils::build_name_path(&node, source)
    }
}
