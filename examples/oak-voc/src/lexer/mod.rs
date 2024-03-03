//! AWSL 词法扫描器（字符级）。

use std::ops::Range;

use super::error::AwslParseError;

/// 顶层模板容器标签（`<widget>` 与 `<template>` 语义等价）。
pub const TOP_LEVEL_TEMPLATE_TAGS: &[&str] = &["widget", "template"];

/// AWSL 词法扫描状态。
pub struct Lexer<'a> {
    /// 源文本。
    pub source: &'a str,
    /// 当前字节偏移。
    pub pos: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source, pos: 0 }
    }

    /// 当前位置是否以某个顶层模板开标签开头（`<widget` / `<template`）。
    pub fn peek_top_level_template_open(&self) -> Option<&'static str> {
        for tag in TOP_LEVEL_TEMPLATE_TAGS {
            if self.peek_starts_with(&format!("<{tag}")) {
                return Some(tag);
            }
        }
        None
    }

    pub fn read_raw_expr_until_gt(&mut self) -> String {
        let start = self.pos;
        let mut depth_paren = 0i32;
        let mut depth_brace = 0i32;
        let mut in_string = None::<char>;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if let Some(quote) = in_string {
                if ch == '\\' {
                    self.pos += 1;
                    if self.pos < self.source.len() {
                        self.pos += self.peek_char().unwrap().len_utf8();
                    }
                    continue;
                }
                if ch == quote {
                    in_string = None;
                }
                self.pos += ch.len_utf8();
                continue;
            }
            if ch == '"' || ch == '\'' {
                in_string = Some(ch);
                self.pos += ch.len_utf8();
                continue;
            }
            if ch == '(' {
                depth_paren += 1;
            }
            else if ch == ')' {
                depth_paren -= 1;
            }
            else if ch == '{' {
                depth_brace += 1;
            }
            else if ch == '}' {
                depth_brace -= 1;
            }
            else if ch == '>' && depth_paren == 0 && depth_brace == 0 && !self.peek_starts_with("/>") {
                break;
            }
            else if self.source[self.pos..].starts_with("/>") && depth_paren == 0 && depth_brace == 0 {
                break;
            }
            self.pos += ch.len_utf8();
        }
        self.source[start..self.pos].trim().to_string()
    }

    pub fn read_attribute_value_raw(&mut self) -> String {
        if self.peek_char_eq('"') || self.peek_char_eq('\'') {
            return self.read_quoted_string().unwrap_or_default();
        }
        if self.peek_char_eq('{') {
            return self.read_braced_expr().unwrap_or_default();
        }
        self.read_unquoted_attr_value()
    }

    pub fn read_braced_expr(&mut self) -> Result<String, AwslParseError> {
        self.expect_char('{')?;
        let mut depth = 1;
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if ch == '{' {
                depth += 1;
            }
            else if ch == '}' {
                depth -= 1;
                if depth == 0 {
                    let expr = self.source[start..self.pos].trim().to_string();
                    self.pos += 1;
                    return Ok(expr);
                }
            }
            self.pos += ch.len_utf8();
        }
        Err(self.error("unclosed `{` in expression"))
    }

    pub fn read_quoted_string(&mut self) -> Result<String, AwslParseError> {
        let quote = self.peek_char().ok_or_else(|| self.error("expected quoted string"))?;
        if quote != '"' && quote != '\'' {
            return Err(self.error("expected quoted string"));
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if ch == quote {
                let value = self.source[start..self.pos].to_string();
                self.pos += 1;
                return Ok(value);
            }
            if ch == '\\' {
                self.pos += 1;
                if self.pos < self.source.len() {
                    self.pos += self.peek_char().unwrap().len_utf8();
                }
                continue;
            }
            self.pos += ch.len_utf8();
        }
        Err(self.error("unclosed string literal"))
    }

    pub fn read_quoted_or_bare(&mut self) -> String {
        if self.peek_char_eq('"') || self.peek_char_eq('\'') {
            return self.read_quoted_string().unwrap_or_default();
        }
        self.read_unquoted_attr_value()
    }

    pub fn read_unquoted_attr_value(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if ch.is_whitespace() || ch == '>' || ch == '/' {
                break;
            }
            self.pos += ch.len_utf8();
        }
        self.source[start..self.pos].to_string()
    }

    pub fn read_name(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                self.pos += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.source[start..self.pos].to_string()
    }

    pub fn read_name_with_colon(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if ch.is_alphanumeric() || ch == '_' || ch == '-' || ch == ':' {
                self.pos += ch.len_utf8();
            }
            else {
                break;
            }
        }
        self.source[start..self.pos].to_string()
    }

    pub fn read_text_until(&mut self, stop: &[char]) -> String {
        let start = self.pos;
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if stop.contains(&ch) {
                break;
            }
            self.pos += ch.len_utf8();
        }
        self.source[start..self.pos].to_string()
    }

    pub fn read_until_literal(&mut self, literal: &str) -> Result<String, AwslParseError> {
        let start = self.pos;
        while self.pos < self.source.len() {
            if self.peek_starts_with(literal) {
                return Ok(self.source[start..self.pos].to_string());
            }
            self.pos += self.peek_char().unwrap().len_utf8();
        }
        Err(self.error(format!("expected `{literal}`")))
    }

    pub fn skip_until(&mut self, ch: char) -> Result<(), AwslParseError> {
        while self.pos < self.source.len() {
            if self.peek_char() == Some(ch) {
                return Ok(());
            }
            self.pos += self.peek_char().unwrap().len_utf8();
        }
        Err(self.error(format!("expected `{ch}`")))
    }

    pub fn skip_trivia(&mut self) {
        while self.pos < self.source.len() {
            let ch = self.peek_char().unwrap();
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            }
            else {
                break;
            }
        }
    }

    pub fn peek_starts_with(&self, literal: &str) -> bool {
        self.source[self.pos..].starts_with(literal)
    }

    pub fn peek_starts_with_ignore_case(&self, literal: &str) -> bool {
        self.source[self.pos..].len() >= literal.len() && self.source[self.pos..self.pos + literal.len()].eq_ignore_ascii_case(literal)
    }

    pub fn peek_char_eq(&self, ch: char) -> bool {
        self.peek_char() == Some(ch)
    }

    pub fn peek_is_name_start(&self) -> bool {
        self.peek_char().is_some_and(|ch| ch.is_alphabetic() || ch == '_')
    }

    pub fn peek_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    pub fn expect_char(&mut self, expected: char) -> Result<(), AwslParseError> {
        if self.peek_char() == Some(expected) {
            self.pos += expected.len_utf8();
            Ok(())
        }
        else {
            Err(self.error(format!("expected `{expected}`")))
        }
    }

    pub fn expect_literal(&mut self, literal: &str) -> Result<(), AwslParseError> {
        if self.peek_starts_with(literal) {
            self.pos += literal.len();
            Ok(())
        }
        else {
            Err(self.error(format!("expected `{literal}`")))
        }
    }

    pub fn span_before(&self, len: usize) -> Range<usize> {
        self.pos - len..self.pos
    }

    pub fn error(&self, message: impl Into<String>) -> AwslParseError {
        self.error_at(self.pos, message)
    }

    pub fn error_at(&self, pos: usize, message: impl Into<String>) -> AwslParseError {
        AwslParseError { message: message.into(), span: pos..pos.saturating_add(1) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn peek_top_level_template_open_recognizes_widget_and_template() {
        let widget = Lexer::new("<widget todo>");
        assert_eq!(widget.peek_top_level_template_open(), Some("widget"));

        let template = Lexer::new("<template App>");
        assert_eq!(template.peek_top_level_template_open(), Some("template"));

        let div = Lexer::new("<div>");
        assert_eq!(div.peek_top_level_template_open(), None);
    }

    #[test]
    fn read_name_reads_tag_and_attr_names() {
        let mut lexer = Lexer::new("TodoItem on:click");
        assert_eq!(lexer.read_name(), "TodoItem");
        lexer.skip_trivia();
        assert_eq!(lexer.read_name_with_colon(), "on:click");
    }

    #[test]
    fn read_quoted_string_supports_single_and_double_quotes() {
        let mut single = Lexer::new("'all'");
        assert_eq!(single.read_quoted_string().unwrap(), "all");

        let mut double = Lexer::new("\"active\"");
        assert_eq!(double.read_quoted_string().unwrap(), "active");
    }

    #[test]
    fn read_braced_expr_handles_nested_braces() {
        let mut lexer = Lexer::new("{filter == 'all' ? 'active' : ''}");
        assert_eq!(lexer.read_braced_expr().unwrap(), "filter == 'all' ? 'active' : ''");
    }

    #[test]
    fn read_raw_expr_until_gt_stops_at_tag_close() {
        let mut lexer = Lexer::new("todos() key=\"item.id\"");
        assert_eq!(lexer.read_raw_expr_until_gt(), "todos() key=\"item.id\"");
    }
}
