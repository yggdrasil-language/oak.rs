//! AWSL 模板解析器。

use super::{
    ast::{AwslAttribute, AwslAttributeValue, AwslDirective, AwslDirectiveKind, AwslElement, AwslImport, AwslRoot, AwslTemplateNode, AwslTextPart},
    error::AwslParseError,
    html::is_html_void_element,
    lexer::Lexer,
};

/// Parsed top-level `<widget>` / `<template>` container body.
struct TopLevelTemplateParts {
    name: Option<String>,
    template: Vec<AwslTemplateNode>,
}

/// AWSL parser.
pub struct AwslParser<'a> {
    lexer: Lexer<'a>,
    strict_mode: bool,
}

impl<'a> AwslParser<'a> {
    /// Parse an AWSL source file.
    pub fn parse_root(source: &'a str) -> Result<AwslRoot, AwslParseError> {
        Self::parse_root_with_options(source, false)
    }

    /// Parse an AWSL source file with options.
    pub fn parse_root_with_options(source: &'a str, strict_mode: bool) -> Result<AwslRoot, AwslParseError> {
        let start = 0;
        let mut parser = Self { lexer: Lexer::new(source), strict_mode };
        parser.lexer.skip_trivia();

        let mut widget_name = None;
        let mut has_widget_shell = false;
        let mut template = Vec::new();
        let mut script = None;
        let mut styles = Vec::new();
        let mut imports = Vec::new();

        if parser.lexer.peek_starts_with("<import:") {
            while parser.lexer.peek_starts_with("<import:") {
                imports.push(parser.parse_import()?);
                parser.lexer.skip_trivia();
            }
        }

        if let Some(tag) = parser.lexer.peek_top_level_template_open() {
            let parts = parser.parse_top_level_template(tag)?;
            has_widget_shell = true;
            widget_name = parts.name;
            template = parts.template;
            parser.lexer.skip_trivia();
        }

        while parser.lexer.pos < source.len() {
            parser.lexer.skip_trivia();
            if parser.lexer.pos >= source.len() {
                break;
            }
            if parser.lexer.peek_starts_with("<script") {
                if script.is_some() {
                    return Err(parser.error("duplicate <script> block"));
                }
                script = Some(parser.parse_block_section("script")?);
            }
            else if parser.lexer.peek_starts_with("<style") {
                styles.push(parser.parse_style_block()?);
            }
            else if let Some(tag) = parser.lexer.peek_top_level_template_open() {
                if has_widget_shell {
                    return Err(parser.error(format!("duplicate top-level <{tag}> block")));
                }
                let parts = parser.parse_top_level_template(tag)?;
                has_widget_shell = true;
                widget_name = parts.name;
                template = parts.template;
            }
            else {
                return Err(parser.error("unexpected content outside widget/template/script/style blocks"));
            }
            parser.lexer.skip_trivia();
        }

        let end = source.len();
        Ok(AwslRoot { has_widget_shell, widget_name, template, script, styles, imports, span: start..end })
    }

    /// Parse a top-level `<widget>` or `<template>` container (equivalent semantics).
    fn parse_top_level_template(&mut self, container_tag: &str) -> Result<TopLevelTemplateParts, AwslParseError> {
        let start = self.lexer.pos;
        self.lexer.expect_literal(&format!("<{container_tag}"))?;
        self.lexer.skip_trivia();

        let name = if self.lexer.peek_is_name_start() && !self.lexer.peek_starts_with(">") && !self.lexer.peek_starts_with("/>") {
            let component_name = self.lexer.read_name();
            self.lexer.skip_trivia();
            Some(component_name)
        }
        else {
            None
        };

        if self.lexer.peek_starts_with("/>") {
            self.lexer.pos += 2;
            return Ok(TopLevelTemplateParts { name, template: Vec::new() });
        }

        self.lexer.expect_char('>')?;
        self.lexer.skip_trivia();

        let close = format!("</{container_tag}>");
        let mut template = Vec::new();

        while !self.lexer.peek_starts_with(&close) {
            if self.lexer.pos >= self.lexer.source.len() {
                return Err(self.error_at(start, format!("unclosed <{container_tag}>")));
            }
            self.lexer.skip_trivia();
            if self.lexer.peek_starts_with(&close) {
                break;
            }
            if self.lexer.peek_starts_with("<script") {
                return Err(self.reject_nested_section_tag("script"));
            }
            else if self.lexer.peek_starts_with("<style") {
                return Err(self.reject_nested_section_tag("style"));
            }
            else if self.lexer.peek_starts_with("<template") {
                return Err(self.reject_nested_section_tag("template"));
            }
            else {
                template.extend(self.parse_node()?);
            }
        }

        self.lexer.expect_literal(&close)?;
        Ok(TopLevelTemplateParts { name, template })
    }

    fn reject_nested_section_tag(&self, tag: &str) -> AwslParseError {
        self.error_at(self.lexer.pos, format!("<{tag}> must be a top-level sibling of <widget>; nesting <script>, <style>, or <template> inside <widget> is not supported"))
    }

    fn parse_import(&mut self) -> Result<AwslImport, AwslParseError> {
        let start = self.lexer.pos;
        self.lexer.expect_literal("<import:")?;
        let name = self.lexer.read_name();
        self.lexer.skip_trivia();
        self.lexer.expect_literal("from")?;
        self.lexer.skip_trivia();
        let from = self.lexer.read_quoted_or_bare();
        self.lexer.skip_trivia();
        self.lexer.expect_literal("/>")?;
        Ok(AwslImport { name, from, span: start..self.lexer.pos })
    }

    fn parse_block_section(&mut self, tag: &str) -> Result<String, AwslParseError> {
        let open = format!("<{tag}");
        let close = format!("</{tag}>");
        self.lexer.expect_literal(&open)?;
        self.lexer.skip_until('>')?;
        self.lexer.expect_char('>')?;
        let body_start = self.lexer.pos;
        let close_pos = self.lexer.source[body_start..].find(&close).ok_or_else(|| self.error(format!("expected `{close}`")))?;
        let body = self.lexer.source[body_start..body_start + close_pos].to_string();
        self.lexer.pos = body_start + close_pos + close.len();
        Ok(body)
    }

    /// Parse a `<style>` block, capturing an optional `target="dom webview"` attribute list.
    fn parse_style_block(&mut self) -> Result<crate::AwslStyleBlock, AwslParseError> {
        self.lexer.expect_literal("<style")?;
        self.lexer.skip_trivia();
        let target = if self.lexer.peek_starts_with("target") {
            self.lexer.expect_literal("target")?;
            self.lexer.skip_trivia();
            self.lexer.expect_char('=')?;
            self.lexer.skip_trivia();
            self.lexer.expect_char('"')?;
            let value_start = self.lexer.pos;
            let value_end = self.lexer.source[value_start..].find('"').ok_or_else(|| self.error("unterminated target attribute"))?;
            let raw = &self.lexer.source[value_start..value_start + value_end];
            self.lexer.pos = value_start + value_end + 1;
            let names: Vec<String> = raw.split_whitespace().map(|name| name.to_string()).collect();
            if names.is_empty() {
                return Err(self.error("<style target> must list at least one backend"));
            }
            Some(names)
        }
        else {
            None
        };
        self.lexer.skip_trivia();
        self.lexer.expect_char('>')?;
        let body_start = self.lexer.pos;
        let close = "</style>";
        let close_pos = self.lexer.source[body_start..].find(close).ok_or_else(|| self.error("expected `</style>`"))?;
        let content = self.lexer.source[body_start..body_start + close_pos].to_string();
        self.lexer.pos = body_start + close_pos + close.len();
        Ok(crate::AwslStyleBlock { target, content })
    }

    fn parse_nodes_until(&mut self, stop_tags: &[&str]) -> Result<Vec<AwslTemplateNode>, AwslParseError> {
        let mut nodes = Vec::new();
        loop {
            self.lexer.skip_trivia();
            if self.lexer.pos >= self.lexer.source.len() {
                break;
            }
            if stop_tags.iter().any(|tag| self.lexer.peek_starts_with(tag)) {
                break;
            }
            if self.lexer.peek_starts_with("</") {
                break;
            }
            nodes.extend(self.parse_node()?);
        }
        Ok(nodes)
    }

    fn parse_node(&mut self) -> Result<Vec<AwslTemplateNode>, AwslParseError> {
        if self.lexer.peek_char_eq('<') {
            let element = self.parse_element()?;
            return Ok(vec![AwslTemplateNode::Element(element)]);
        }
        let start = self.lexer.pos;
        let text = self.lexer.read_text_until(&['<']);
        if text.is_empty() {
            return Err(self.error_at(start, "empty template region: expected `<` element or interpolation"));
        }
        self.text_template_nodes(&text, start)
    }

    /// 文本区域的模板拆分（合同 §4.10 D/F）：`{ expr }` 是插值，`\{`/`\}` 是原样
    /// 花括号的转义候选（成对留在文本段，解码归消费方），裸 `}` 结构化报错。
    /// 表达式边界与语法由共享的 Valkyrie 表达式解析器给出，AWSL 不自建花括号计数。
    fn text_template_nodes(&self, content: &str, content_start: usize) -> Result<Vec<AwslTemplateNode>, AwslParseError> {
        if !content.contains('{') && !content.contains('}') {
            return Ok(vec![AwslTemplateNode::Text { content: content.to_string(), span: content_start..content_start + content.len() }]);
        }
        let segments = self.template_segments(content, content_start)?;
        Ok(self.segments_to_template_nodes(&segments, content, content_start))
    }

    /// 请求共享模板解析器对捕获区间做有边界分段。
    fn template_segments(&self, content: &str, content_start: usize) -> Result<Vec<oak_valkyrie::ast::StringSegment>, AwslParseError> {
        let language = oak_valkyrie::ValkyrieLanguage::default();
        oak_valkyrie::ValkyrieBuilder::new(&language)
            .parse_quote_template(content, content_start)
            .map_err(|error| self.error_at(content_start, error.to_string()))
    }

    fn segments_to_template_nodes(&self, segments: &[oak_valkyrie::ast::StringSegment], content: &str, content_start: usize) -> Vec<AwslTemplateNode> {
        segments
            .iter()
            .map(|segment| match segment {
                oak_valkyrie::ast::StringSegment::Text(text) => AwslTemplateNode::Text { content: text.content.clone(), span: text.span.start..text.span.end },
                oak_valkyrie::ast::StringSegment::Interpolation(interpolation) => AwslTemplateNode::Interpolation {
                    expr: interpolation_source(content, content_start, interpolation),
                    span: interpolation.span.start..interpolation.span.end,
                },
            })
            .collect()
    }

    /// 引号属性内容的模板值：与文本区域同一套规则（§4.10 F）。
    /// `<tag="{ expr }">` 是表达式插值，不是 dict，也不是 JavaScript 模板。
    fn attribute_template_value(&self, content: &str, content_start: usize) -> Result<AwslAttributeValue, AwslParseError> {
        if !content.contains('{') && !content.contains('}') {
            return Ok(AwslAttributeValue::Literal(content.to_string()));
        }
        let segments = self.template_segments(content, content_start)?;
        Ok(self.segments_to_attr_value(&segments, content, content_start))
    }

    fn segments_to_attr_value(&self, segments: &[oak_valkyrie::ast::StringSegment], content: &str, content_start: usize) -> AwslAttributeValue {
        let parts: Vec<AwslTextPart> = segments
            .iter()
            .map(|segment| match segment {
                oak_valkyrie::ast::StringSegment::Text(text) => AwslTextPart::Text(text.content.clone()),
                oak_valkyrie::ast::StringSegment::Interpolation(interpolation) => {
                    AwslTextPart::Expr(interpolation_source(content, content_start, interpolation))
                }
            })
            .collect();
        match parts.as_slice() {
            [AwslTextPart::Text(text)] => AwslAttributeValue::Literal(text.clone()),
            [AwslTextPart::Expr(expr)] => AwslAttributeValue::Expression(expr.clone()),
            _ => AwslAttributeValue::Mixed(parts),
        }
    }

    fn parse_element(&mut self) -> Result<AwslElement, AwslParseError> {
        let start = self.lexer.pos;
        self.lexer.expect_char('<')?;
        let tag = self.lexer.read_name();
        self.lexer.skip_trivia();

        if matches!(tag.as_str(), "widget" | "template") {
            return Err(self.error_at(start, format!("<{tag}> is a top-level template container; use it at file root, not nested inside markup")));
        }
        if matches!(tag.as_str(), "script" | "style") {
            return Err(self.reject_nested_section_tag(&tag));
        }

        let mut attributes = Vec::new();
        let mut directives = Vec::new();

        if matches!(tag.as_str(), "if" | "loop") {
            if let Some(header_attrs) = self.try_parse_control_flow_header(&tag)? {
                attributes.extend(header_attrs);
            }
        }

        while !self.lexer.peek_char_eq('>') && !self.lexer.peek_starts_with("/>") {
            if self.lexer.peek_char_eq('@') {
                directives.push(self.parse_directive()?);
            }
            else if self.lexer.peek_char_eq(':') {
                attributes.push(self.parse_colon_binding_attribute()?);
            }
            else if self.lexer.peek_starts_with("on:") {
                return Err(self.error("`on:click` is not HTML; use `@click=\"handler\"` or `:on_click=\"handler\"` on components"));
            }
            else if self.lexer.peek_is_name_start() {
                attributes.push(self.parse_static_attribute(&tag)?);
            }
            else {
                break;
            }
            self.lexer.skip_trivia();
        }

        let self_closing = if self.lexer.peek_starts_with("/>") {
            self.lexer.pos += 2;
            true
        }
        else {
            self.lexer.expect_char('>')?;
            false
        };

        if !self_closing && is_html_void_element(&tag) {
            self.lexer.skip_trivia();
            self.try_consume_void_close_tag(&tag);
            return Ok(AwslElement { tag, attributes, directives, children: Vec::new(), self_closing: true, span: start..self.lexer.pos });
        }

        let mut children = Vec::new();
        if !self_closing {
            let close = format!("</{tag}>");
            while !self.lexer.peek_starts_with(&close) {
                if self.lexer.pos >= self.lexer.source.len() {
                    return Err(self.error_at(start, format!("unclosed <{tag}>")));
                }
                if self.lexer.peek_starts_with("</") {
                    return Err(self.error_at(self.lexer.pos, format!("unexpected closing tag while parsing <{tag}>")));
                }
                children.extend(self.parse_node()?);
            }
            self.lexer.expect_literal(&close)?;
        }

        Ok(AwslElement { tag, attributes, directives, children, self_closing, span: start..self.lexer.pos })
    }

    /// 吞掉可选的 `</void-tag>`（HTML 粘贴兼容）。
    fn try_consume_void_close_tag(&mut self, tag: &str) {
        let close = format!("</{tag}>");
        if self.lexer.peek_starts_with_ignore_case(&close) {
            self.lexer.pos += close.len();
        }
    }

    fn parse_directive(&mut self) -> Result<AwslDirective, AwslParseError> {
        let start = self.lexer.pos;
        self.lexer.expect_char('@')?;
        let name = self.lexer.read_name();
        self.lexer.skip_trivia();

        let (kind, value) = match name.as_str() {
            "if" => (AwslDirectiveKind::If, Some(self.parse_quoted_dsl_value("@if")?)),
            "for" => {
                return Err(self.error("@for is not supported; use `@loop=\"item in expr\"` or `<loop item in expr>`"));
            }
            "loop" => (AwslDirectiveKind::Loop, Some(self.parse_quoted_dsl_value("@loop")?)),
            "bind" => (AwslDirectiveKind::Bind, Some(self.parse_quoted_dsl_value("@bind")?)),
            "style" => (AwslDirectiveKind::Style, Some(self.parse_quoted_dsl_value("@style")?)),
            "ref" => (AwslDirectiveKind::Ref, Some(self.parse_quoted_dsl_value("@ref")?)),
            "class" => (AwslDirectiveKind::Class, Some(self.parse_quoted_dsl_value("@class")?)),
            event if event.starts_with("on") => {
                let event_name = event.strip_prefix("on").unwrap_or(event).trim_start_matches(':').to_string();
                (AwslDirectiveKind::On(event_name), Some(self.parse_quoted_dsl_value(&format!("@{event}"))?))
            }
            event => (AwslDirectiveKind::On(event.to_string()), Some(self.parse_quoted_dsl_value(&format!("@{event}"))?)),
        };

        Ok(AwslDirective { kind, value, span: start..self.lexer.pos })
    }

    fn parse_quoted_dsl_value(&mut self, context: &str) -> Result<String, AwslParseError> {
        self.lexer.skip_trivia();
        if !self.lexer.peek_char_eq('=') {
            return Err(self.error(format!("{context} requires `=\"expr\"`")));
        }
        self.lexer.pos += 1;
        self.lexer.skip_trivia();
        if self.lexer.peek_char_eq('{') {
            return Err(self.error(format!("{context} uses quoted DSL `=\"...\"`; braced `={{...}}` is not supported")));
        }
        if self.lexer.peek_char_eq('"') || self.lexer.peek_char_eq('\'') {
            return self.lexer.read_quoted_string();
        }
        Err(self.error(format!("{context} requires a quoted expression `\"...\"` or `'...'`")))
    }

    fn parse_if_directive_value(&mut self) -> Result<String, AwslParseError> {
        self.parse_quoted_dsl_value("@if")
    }

    fn parse_optional_directive_value(&mut self) -> Result<Option<String>, AwslParseError> {
        self.lexer.skip_trivia();
        if self.lexer.peek_char_eq('=') {
            return Ok(Some(self.parse_quoted_dsl_value("directive")?));
        }
        Ok(None)
    }

    fn parse_colon_binding_attribute(&mut self) -> Result<AwslAttribute, AwslParseError> {
        let start = self.lexer.pos;
        self.lexer.expect_char(':')?;
        let name = self.lexer.read_name();
        self.lexer.skip_trivia();
        let value = AwslAttributeValue::Expression(self.parse_quoted_dsl_value(&format!(":{name}"))?);
        Ok(AwslAttribute { name, value, span: start..self.lexer.pos })
    }

    fn parse_static_attribute(&mut self, _tag: &str) -> Result<AwslAttribute, AwslParseError> {
        let start = self.lexer.pos;
        let name = self.lexer.read_name();
        self.lexer.skip_trivia();
        let value = if self.lexer.peek_char_eq('=') {
            self.lexer.pos += 1;
            self.lexer.skip_trivia();
            if self.lexer.peek_char_eq('{') {
                // 控制流表达式属性（`each`、`key`）允许花括号表达式值，产出 `Expression`。
                if matches!(name.as_str(), "each" | "key" | "condition") {
                    AwslAttributeValue::Expression(self.lexer.read_braced_expr()?)
                }
                else {
                    return Err(self.error(format!("attribute `{name}` is static; use `=\"...\"`, not `={{...}}`")));
                }
            }
            else if self.lexer.peek_char_eq('"') || self.lexer.peek_char_eq('\'') {
                // 引号内容按共享模板规则处理：`{ expr }` 插值、`\{`/`\}` 原样花括号（§4.10 F）。
                let (raw, raw_start) = self.lexer.read_quoted_string_spanned()?;
                self.attribute_template_value(&raw, raw_start)?
            }
            else {
                return Err(self.error(format!("attribute `{name}` requires a quoted string literal `\"...\"`")));
            }
        }
        else if !self.strict_mode {
            AwslAttributeValue::Literal(String::new())
        }
        else {
            return Err(self.error_at(start, format!("attribute `{name}` requires a value in strict mode")));
        };
        Ok(AwslAttribute { name, value, span: start..self.lexer.pos })
    }

    fn try_parse_control_flow_header(&mut self, tag: &str) -> Result<Option<Vec<AwslAttribute>>, AwslParseError> {
        self.lexer.skip_trivia();
        if self.lexer.peek_char_eq('>') || self.lexer.peek_starts_with("/>") || self.lexer.peek_char_eq('@') || self.lexer.peek_char_eq(':') {
            return Ok(None);
        }
        if tag == "loop" {
            if let Some(attrs) = self.try_parse_loop_in_header() {
                return Ok(Some(attrs));
            }
        }
        if tag == "if" {
            if let Some(condition) = self.try_parse_bare_if_condition() {
                let span = self.lexer.pos..self.lexer.pos;
                return Ok(Some(vec![AwslAttribute { name: "condition".into(), value: AwslAttributeValue::Expression(condition), span }]));
            }
        }
        Ok(None)
    }

    fn try_parse_loop_in_header(&mut self) -> Option<Vec<AwslAttribute>> {
        let checkpoint = self.lexer.pos;
        self.lexer.skip_trivia();
        if !self.lexer.peek_is_name_start() {
            return None;
        }
        let item = self.lexer.read_name();
        self.lexer.skip_trivia();
        if !self.lexer.source[self.lexer.pos..].starts_with("in") {
            self.lexer.pos = checkpoint;
            return None;
        }
        let in_kw = self.lexer.read_name();
        if in_kw != "in" {
            self.lexer.pos = checkpoint;
            return None;
        }
        self.lexer.skip_trivia();
        let items_expr = self.lexer.read_raw_expr_until_gt();
        let span = checkpoint..self.lexer.pos;
        Some(vec![AwslAttribute { name: "item".into(), value: AwslAttributeValue::Literal(item.clone()), span: span.clone() }, AwslAttribute { name: "each".into(), value: AwslAttributeValue::Expression(items_expr), span }])
    }

    fn try_parse_bare_if_condition(&mut self) -> Option<String> {
        let checkpoint = self.lexer.pos;
        self.lexer.skip_trivia();
        if self.lexer.peek_char_eq('>') || self.lexer.peek_starts_with("/>") {
            return None;
        }
        if self.lexer.peek_is_name_start() {
            self.lexer.read_name();
            self.lexer.skip_trivia();
            if self.lexer.peek_char_eq('=') {
                self.lexer.pos = checkpoint;
                return None;
            }
            self.lexer.pos = checkpoint;
        }
        let expr = self.lexer.read_raw_expr_until_gt();
        if expr.trim().is_empty() {
            self.lexer.pos = checkpoint;
            None
        }
        else {
            Some(expr)
        }
    }

    fn error(&self, message: impl Into<String>) -> AwslParseError {
        self.lexer.error(message)
    }

    fn error_at(&self, pos: usize, message: impl Into<String>) -> AwslParseError {
        self.lexer.error_at(pos, message)
    }
}

/// `{ ... }` 区间的表达式源码（不含外层花括号）；区间由共享模板解析器冻结。
fn interpolation_source(content: &str, content_start: usize, interpolation: &oak_valkyrie::ast::InterpolationSegment) -> String {
    let inner_start = interpolation.span.start + 1 - content_start;
    let inner_end = interpolation.span.end - 1 - content_start;
    content[inner_start..inner_end].trim().to_string()
}
