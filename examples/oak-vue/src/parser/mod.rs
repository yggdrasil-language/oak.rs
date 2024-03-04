/// Element types for the Vue language.
pub mod element_type;

use crate::{
    language::VueLanguage,
    lexer::{VueLexer, token_type::VueTokenType},
};
use oak_core::{
    GreenNode, TextEdit,
    parser::{
        ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer,
        pratt::{Associativity, Pratt, PrattParser, unary},
    },
    source::{Source, SourceText},
};

/// Parser state type alias.
pub(crate) type State<'a, S> = ParserState<'a, VueLanguage, S>;

/// HTML void elements (lowercase author tags only).
///
/// PascalCase tags such as `Link` are Vue components and must parse children.
fn is_html_void_tag(raw: &str) -> bool {
    if raw.chars().next().is_some_and(|c| c.is_ascii_uppercase()) {
        return false;
    }
    matches!(raw, "area" | "base" | "br" | "col" | "embed" | "hr" | "img" | "input" | "link" | "meta" | "param" | "source" | "track" | "wbr")
}

/// Vue parser.
pub struct VueParser<'config> {
    pub(crate) config: &'config VueLanguage,
}

impl<'config> Parser<VueLanguage> for VueParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<VueLanguage>) -> ParseOutput<'a, VueLanguage> {
        let lexer = VueLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let cp = state.checkpoint();
            while state.not_at_end() {
                if state.at(VueTokenType::TemplateStart) {
                    self.parse_special_block(state, VueTokenType::TemplateStart, "template", true);
                }
                else if state.at(VueTokenType::ScriptStart) {
                    self.parse_special_block(state, VueTokenType::ScriptStart, "script", false);
                }
                else if state.at(VueTokenType::StyleStart) {
                    self.parse_special_block(state, VueTokenType::StyleStart, "style", false);
                }
                else {
                    self.parse_node(state);
                }
            }
            Ok(state.finish_at(cp, crate::parser::element_type::VueElementType::Root))
        })
    }
}

#[allow(dead_code)]
impl<'config> VueParser<'config> {
    /// Creates a new Vue parser with the given configuration.
    pub fn new(config: &'config VueLanguage) -> Self {
        Self { config }
    }

    /// Parses an expression.
    pub fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, VueLanguage> {
        PrattParser::parse(state, 0, self)
    }

    /// Parses only an expression.
    pub fn parse_expression_only<'a, S: Source + ?Sized>(&self, source: &'a S, session: &'a mut oak_core::parser::ParseSession<VueLanguage>) -> oak_core::parser::ParseOutput<'a, VueLanguage> {
        let lexer = VueLexer::new(self.config);
        oak_core::parser::parse_with_lexer(&lexer, source, &[], session, |state| {
            let cp = state.checkpoint();
            self.parse_expression(state);
            Ok(state.finish_at(cp, crate::parser::element_type::VueElementType::Root))
        })
    }

    /// Parses only a `v-for` expression.
    pub fn parse_v_for_only<'a, S: Source + ?Sized>(&self, source: &'a S, session: &'a mut oak_core::parser::ParseSession<VueLanguage>) -> oak_core::parser::ParseOutput<'a, VueLanguage> {
        let lexer = VueLexer::new(self.config);
        oak_core::parser::parse_with_lexer(&lexer, source, &[], session, |state| {
            let cp = state.checkpoint();
            self.parse_v_for_expression(state);
            Ok(state.finish_at(cp, crate::parser::element_type::VueElementType::Root))
        })
    }

    fn parse_special_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, start_kind: VueTokenType, tag_name: &str, recursive: bool) {
        let cp = state.checkpoint();
        let tag_cp = state.checkpoint();
        state.expect(start_kind).ok();

        while state.not_at_end() && !state.at(VueTokenType::Gt) {
            if state.at(VueTokenType::Whitespace) {
                state.bump();
                continue;
            }
            self.parse_attribute(state);
        }

        state.expect(VueTokenType::Gt).ok();
        let tag_node = state.finish_at(tag_cp, crate::parser::element_type::VueElementType::Tag);
        state.sink.restore(tag_cp.1);
        state.push_child(tag_node);

        if tag_name == "script" {
            self.parse_program(state);
        }
        else if recursive {
            while state.not_at_end() && !state.at(VueTokenType::LtSlash) {
                self.parse_node(state);
            }
        }
        else {
            // Raw content
            while state.not_at_end() && !state.at(VueTokenType::LtSlash) {
                state.bump();
            }
        }

        if state.at(VueTokenType::LtSlash) {
            let close_cp = state.checkpoint();
            state.expect(VueTokenType::LtSlash).ok();
            if state.at(VueTokenType::Identifier) {
                state.bump();
            }
            else {
                let _ = Self::bump_attr_name(state);
            }
            state.expect(VueTokenType::Gt).ok();
            let close_node = state.finish_at(close_cp, crate::parser::element_type::VueElementType::CloseTag);
            state.sink.restore(close_cp.1);
            state.push_child(close_node);
        }

        let block_kind = match tag_name {
            "template" => crate::parser::element_type::VueElementType::TemplateElement,
            "script" => crate::parser::element_type::VueElementType::Program,
            _ => crate::parser::element_type::VueElementType::Element,
        };
        let node = state.finish_at(cp, block_kind);
        state.sink.restore(cp.1);
        state.push_child(node);
    }

    fn parse_node<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if state.at(VueTokenType::Lt) || state.at(VueTokenType::ScriptStart) || state.at(VueTokenType::StyleStart) || state.at(VueTokenType::TemplateStart) || state.at(VueTokenType::DocTypeStart) {
            self.parse_element(state);
        }
        else if state.at(VueTokenType::InterpolationStart) {
            self.parse_interpolation(state);
        }
        else if state.at(VueTokenType::Comment) {
            let cp = state.checkpoint();
            state.bump();
            let node = state.finish_at(cp, crate::parser::element_type::VueElementType::CommentNode);
            state.sink.restore(cp.1);
            state.push_child(node);
        }
        else if state.at(VueTokenType::LtSlash) {
            // This is a closing tag at a position where we expect a node.
            // We should NOT consume it as text. The caller (parse_element) will handle it.
            return;
        }
        else {
            let cp = state.checkpoint();
            while state.not_at_end()
                && !state.at(VueTokenType::Lt)
                && !state.at(VueTokenType::LtSlash)
                && !state.at(VueTokenType::ScriptStart)
                && !state.at(VueTokenType::StyleStart)
                && !state.at(VueTokenType::TemplateStart)
                && !state.at(VueTokenType::DocTypeStart)
                && !state.at(VueTokenType::InterpolationStart)
            {
                state.bump();
            }
            let node = state.finish_at(cp, crate::parser::element_type::VueElementType::TextNode);
            state.sink.restore(cp.1);
            state.push_child(node);
        }
    }

    fn parse_element<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        let tag_cp = state.checkpoint();

        let mut tag_name = String::new();
        let mut _start_kind = VueTokenType::Lt;

        if state.at(VueTokenType::Lt) {
            state.expect(VueTokenType::Lt).ok();
            if state.at(VueTokenType::Identifier) {
                tag_name = state.peek_text().unwrap_or_default().to_string();
                state.bump();
            }
        }
        else if state.at(VueTokenType::ScriptStart) {
            _start_kind = VueTokenType::ScriptStart;
            state.bump();
            tag_name = "script".to_string();
        }
        else if state.at(VueTokenType::StyleStart) {
            _start_kind = VueTokenType::StyleStart;
            state.bump();
            tag_name = "style".to_string();
        }
        else if state.at(VueTokenType::TemplateStart) {
            _start_kind = VueTokenType::TemplateStart;
            state.bump();
            tag_name = "template".to_string();
        }
        else if state.at(VueTokenType::DocTypeStart) {
            // Special handling for DOCTYPE
            state.bump();
            while state.not_at_end() && !state.at(VueTokenType::Gt) {
                state.bump();
            }
            state.expect(VueTokenType::Gt).ok();
            let node = state.finish_at(cp, crate::parser::element_type::VueElementType::Element);
            state.sink.restore(cp.1);
            state.push_child(node);
            return;
        }

        // Nested `<template>` / `<script>` / `<style>` are ordinary elements here.
        // Top-level SFC blocks are handled by `parse_root` → `parse_special_block`.
        // Do **not** sink.restore after consuming TemplateStart/ScriptStart/StyleStart:
        // that drops the keyword leaf while the lexer has already advanced, so Tag
        // attributes (#slot) get wrong source offsets and tag_name stays empty.

        while state.not_at_end() && !state.at(VueTokenType::Gt) && !state.at(VueTokenType::SlashGt) {
            if state.at(VueTokenType::Whitespace) {
                state.bump();
                continue;
            }
            let before = state.checkpoint().0;
            self.parse_attribute(state);
            // Unknown / unconsumed tokens must not spin forever allocating green nodes.
            if state.checkpoint().0 == before {
                state.bump();
            }
        }

        let mut is_self_closing = false;
        if state.at(VueTokenType::SlashGt) {
            state.bump();
            is_self_closing = true;
        }
        else {
            state.expect(VueTokenType::Gt).ok();
        }

        let tag_node = state.finish_at(tag_cp, crate::parser::element_type::VueElementType::Tag);
        state.sink.restore(tag_cp.1);
        state.push_child(tag_node);

        if !is_self_closing && !is_html_void_tag(&tag_name) {
            while state.not_at_end() && !state.at(VueTokenType::LtSlash) {
                self.parse_node(state);
            }

            if state.at(VueTokenType::LtSlash) {
                let close_cp = state.checkpoint();
                state.expect(VueTokenType::LtSlash).ok();
                let close_name = if state.at(VueTokenType::Identifier) {
                    let name = state.peek_text().map(|s| s.to_string()).unwrap_or_default();
                    state.bump();
                    name
                }
                else if let Some(name) = Self::bump_attr_name(state) {
                    // Keywords used as tag names (rare) still need consumption.
                    name
                }
                else {
                    String::new()
                };
                state.expect(VueTokenType::Gt).ok();
                if !tag_name.is_empty() && !close_name.is_empty() && !tag_name.eq_ignore_ascii_case(&close_name) {
                    let _ = state.syntax_error(format!("mismatched closing tag: expected `</{tag_name}>`, found `</{close_name}>`"));
                }
                let close_node = state.finish_at(close_cp, crate::parser::element_type::VueElementType::CloseTag);
                state.sink.restore(close_cp.1);
                state.push_child(close_node);
            }
        }

        let node = state.finish_at(cp, crate::parser::element_type::VueElementType::Element);
        state.sink.restore(cp.1);
        state.push_child(node);
    }

    fn parse_attribute<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        let mut is_directive = false;
        let mut directive_name = String::new();

        if state.at(VueTokenType::Colon) {
            state.bump();
            is_directive = true;
            directive_name = "v-bind".to_string();
        }
        else if state.at(VueTokenType::At) {
            state.bump();
            is_directive = true;
            directive_name = "v-on".to_string();
        }
        else if state.at(VueTokenType::Hash) {
            state.bump();
            is_directive = true;
            directive_name = "v-slot".to_string();
        }

        let name_cp = state.checkpoint();
        let name_end = state.current().map(|token| token.span.end);
        // Lexer classifies bare words like `for`/`in`/`as` as keywords. HTML still uses them
        // as attribute names (`:for`, `for=`). Treat those as name tokens, otherwise the
        // attribute scan never advances and arena allocation spins without bound.
        let attribute_name = Self::bump_attr_name(state);
        if let Some(text) = attribute_name.as_deref() {
            if text.starts_with("v-") {
                is_directive = true;
                directive_name = text.to_string();
            }
        }

        let adjacent_colon = state.at(VueTokenType::Colon) && name_end == state.current().map(|token| token.span.start);
        if !is_directive && adjacent_colon {
            is_directive = true;
            directive_name = attribute_name.unwrap_or_default();
        }

        if is_directive && (adjacent_colon || state.at(VueTokenType::LeftBracket)) {
            self.parse_optional_directive_arg(state);
        }

        while state.at(VueTokenType::Dot) {
            let mod_cp = state.checkpoint();
            state.bump();
            let _ = Self::bump_attr_name(state);
            state.finish_at(mod_cp, crate::parser::element_type::VueElementType::Modifier);
        }
        state.finish_at(name_cp, crate::parser::element_type::VueElementType::AttributeName);

        if state.eat(VueTokenType::Eq) {
            let val_cp = state.checkpoint();
            if is_directive {
                self.parse_directive_value(state, &directive_name);
            }
            else if state.at(VueTokenType::StringLiteral) {
                state.bump();
            }
            else {
                self.parse_expression(state);
            }
            state.finish_at(val_cp, crate::parser::element_type::VueElementType::AttributeValue);
        }

        let kind = if is_directive { crate::parser::element_type::VueElementType::Directive } else { crate::parser::element_type::VueElementType::Attribute };
        state.finish_at(cp, kind);

        // Last-resort progress: empty attribute nodes must not be emitted forever.
        if state.checkpoint().0 == cp.0 && state.not_at_end() {
            state.bump();
        }
    }

    /// Bump one attribute/directive name token (`Identifier` or keyword used as HTML name).
    fn bump_attr_name<'a, S: Source + ?Sized>(state: &mut State<'a, S>) -> Option<String> {
        use VueTokenType::*;
        if state.at(Identifier) {
            let text = state.peek_text().map(|c| c.to_string()).unwrap_or_default();
            state.bump();
            return Some(text);
        }
        // Keywords that appear as HTML / Vue attribute names in the wild.
        if matches!(state.peek_kind(), Some(For | In | Of | As | If | Else | From | Default | Const | Let | Var | Function | Return | Break | Continue | Switch | Try | Throw | True | False | Null | Import | Export | While)) {
            let text = state.peek_text().map(|c| c.to_string()).unwrap_or_default();
            state.bump();
            return Some(text);
        }
        None
    }

    /// Static identifier or `[expr]` argument after a directive prefix (`:`, `@`, `#`, `v-*:`).
    fn parse_optional_directive_arg<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        if state.at(VueTokenType::Colon) {
            state.bump();
        }
        if state.at(VueTokenType::LeftBracket) {
            let arg_cp = state.checkpoint();
            state.bump();
            while state.not_at_end() && !state.at(VueTokenType::RightBracket) {
                state.bump();
            }
            state.expect(VueTokenType::RightBracket).ok();
            let node = state.finish_at(arg_cp, crate::parser::element_type::VueElementType::Identifier);
            state.sink.restore(arg_cp.1);
            state.push_child(node);
        }
        else if state.at(VueTokenType::Identifier) || Self::bump_attr_name_is_keyword(state.peek_kind()) {
            let arg_cp = state.checkpoint();
            state.bump();
            let node = state.finish_at(arg_cp, crate::parser::element_type::VueElementType::Identifier);
            state.sink.restore(arg_cp.1);
            state.push_child(node);
        }
    }

    fn bump_attr_name_is_keyword(kind: Option<VueTokenType>) -> bool {
        use VueTokenType::*;
        matches!(kind, Some(For | In | Of | As | If | Else | From | Default | Const | Let | Var | Function | Return | Break | Continue | Switch | Try | Throw | True | False | Null | Import | Export | While))
    }

    fn parse_directive_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, directive_name: &str) {
        if !state.at(VueTokenType::StringLiteral) {
            return;
        }
        if let Some(text) = state.peek_text() {
            if text.len() >= 2 {
                let inner_owned = text[1..text.len() - 1].to_string();
                let inner_source = SourceText::new(inner_owned);
                let mut sub_session = oak_core::parser::ParseSession::default();
                let _parsed = if directive_name == "v-for" { self.parse_v_for_only(&inner_source, &mut sub_session) } else { self.parse_expression_only(&inner_source, &mut sub_session) };
            }
        }
        state.bump();
    }

    /// Parses a `v-for` expression.
    pub fn parse_v_for_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        self.skip_whitespace(state);

        // Parse the left side (pattern)
        let pattern_cp = state.checkpoint();
        if state.at(VueTokenType::LeftParen) {
            state.bump();
            self.skip_whitespace(state);
            state.expect(VueTokenType::Identifier).ok();
            self.skip_whitespace(state);
            if state.eat(VueTokenType::Comma) {
                self.skip_whitespace(state);
                state.expect(VueTokenType::Identifier).ok();
                self.skip_whitespace(state);
            }
            state.expect(VueTokenType::RightParen).ok();
        }
        else {
            state.expect(VueTokenType::Identifier).ok();
        }
        state.finish_at(pattern_cp, crate::parser::element_type::VueElementType::Pattern);

        self.skip_whitespace(state);

        // Parse "in" or "of"
        let kind = if state.at(VueTokenType::In) {
            state.bump();
            VueTokenType::ForInExpr
        }
        else if state.at(VueTokenType::Of) {
            state.bump();
            VueTokenType::ForOfExpr
        }
        else {
            // Fallback if neither "in" nor "of" is found
            state.finish_at(cp, crate::parser::element_type::VueElementType::Error);
            return;
        };

        self.skip_whitespace(state);

        // Parse the right side (expression)
        let expr = self.parse_expression(state);
        state.push_child(expr);

        let element_kind = match kind {
            VueTokenType::ForInExpr => crate::parser::element_type::VueElementType::ForInExpr,
            VueTokenType::ForOfExpr => crate::parser::element_type::VueElementType::ForOfExpr,
            _ => crate::parser::element_type::VueElementType::Error,
        };
        state.finish_at(cp, element_kind);
    }

    fn parse_interpolation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let cp = state.checkpoint();
        state.expect(VueTokenType::InterpolationStart).ok();
        self.skip_whitespace(state);
        let expr = self.parse_expression(state);
        state.push_child(expr);
        self.skip_whitespace(state);
        state.expect(VueTokenType::InterpolationEnd).ok();
        state.finish_at(cp, crate::parser::element_type::VueElementType::Interpolation);
    }

    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.at(VueTokenType::Whitespace) {
            state.bump();
        }
    }

    fn parse_program<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        // Raw-consume script body until `</script>`. Do not parse JS as template nodes —
        // otherwise root-level `parse_node` can hang or mis-structure SFC walks.
        while state.not_at_end() && !state.at(VueTokenType::LtSlash) {
            state.bump();
        }
    }
}

impl<'config> Pratt<VueLanguage> for VueParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, VueLanguage> {
        use VueTokenType::*;
        let cp = state.checkpoint();
        self.skip_whitespace(state);
        let node = if state.at(LeftParen) {
            state.expect(LeftParen).ok();
            if !state.at(RightParen) {
                let inner = PrattParser::parse(state, 0, self);
                state.push_child(inner);
            }
            state.expect(RightParen).ok();
            state.finish_at(cp, crate::parser::element_type::VueElementType::Expression)
        }
        else if state.at(LeftBrace) {
            state.expect(LeftBrace).ok();
            while state.not_at_end() && !state.at(RightBrace) {
                let property_cp = state.checkpoint();
                if state.at(Identifier) || matches!(state.peek_kind(), Some(True | False | Null)) {
                    state.bump();
                    if state.at(Colon) {
                        state.bump();
                        let value = PrattParser::parse(state, 0, self);
                        state.push_child(value);
                    }
                    state.finish_at(property_cp, crate::parser::element_type::VueElementType::ObjectProperty);
                }
                else {
                    state.bump();
                }
                if state.at(Comma) {
                    state.bump();
                }
            }
            state.expect(RightBrace).ok();
            state.finish_at(cp, crate::parser::element_type::VueElementType::ObjectExpr)
        }
        else if state.at(Identifier) {
            state.expect(Identifier).ok();
            state.finish_at(cp, crate::parser::element_type::VueElementType::Identifier)
        }
        else if matches!(state.peek_kind(), Some(True | False | Null)) {
            state.bump();
            state.finish_at(cp, crate::parser::element_type::VueElementType::Literal)
        }
        else if state.at(StringLiteral) {
            state.expect(StringLiteral).ok();
            state.finish_at(cp, crate::parser::element_type::VueElementType::Literal)
        }
        else if state.at(NumberLiteral) {
            state.expect(NumberLiteral).ok();
            state.finish_at(cp, crate::parser::element_type::VueElementType::Literal)
        }
        else {
            state.bump();
            state.finish_at(cp, crate::parser::element_type::VueElementType::Error)
        };

        state.sink.restore(cp.1);
        node
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, VueLanguage> {
        use VueTokenType::*;
        let start_cp = state.checkpoint();
        self.skip_whitespace(state);
        let kind = match state.peek_kind() {
            Some(k) => k,
            None => {
                state.restore(start_cp);
                return self.primary(state);
            }
        };

        match kind {
            Bang | Minus => {
                state.restore(start_cp);
                let cp = state.checkpoint();
                self.skip_whitespace(state);
                let node = unary(state, kind.into(), 90, UnaryExpr.into(), |s, p| PrattParser::parse(s, p, self));
                state.sink.restore(cp.1);
                node
            }
            _ => {
                state.restore(start_cp);
                self.primary(state)
            }
        }
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, VueLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, VueLanguage>> {
        use VueTokenType::*;
        let start_cp = state.checkpoint();
        self.skip_whitespace(state);
        let kind = match state.peek_kind() {
            Some(k) => k,
            None => {
                state.restore(start_cp);
                return None;
            }
        };

        let (prec, assoc) = match kind {
            Dot | LeftBracket | LeftParen => (100, Associativity::Left),
            Star | Slash => (80, Associativity::Left),
            Plus | Minus => (70, Associativity::Left),
            Lt | Gt | LtEq | GtEq | NotEq => (60, Associativity::Left),
            EqEq => (50, Associativity::Left),
            And => (40, Associativity::Left),
            Or => (30, Associativity::Left),
            // Ternary `cond ? then : else` — between logical OR and assignment.
            Question => (20, Associativity::Right),
            Arrow => (5, Associativity::Right),
            Eq => (10, Associativity::Right),
            _ => {
                state.restore(start_cp);
                return None;
            }
        };

        if prec < min_precedence {
            state.restore(start_cp);
            return None;
        }

        // Re-position left and whitespace:
        // 1. Restore to before whitespace
        state.restore(start_cp);
        // 2. Take checkpoint for the new infix node
        let op_cp = state.checkpoint();
        // 3. Push left as the first child
        state.push_child(left);
        // 4. Re-skip whitespace so it becomes a child after left
        self.skip_whitespace(state);

        let node = match kind {
            Arrow => {
                state.expect(Arrow).ok();
                self.skip_whitespace(state);
                let body = PrattParser::parse(state, 0, self);
                state.push_child(body);
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::ArrowFunction)
            }
            Dot => {
                state.expect(Dot).ok();
                self.skip_whitespace(state);
                state.expect(Identifier).ok();
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::MemberExpr)
            }
            LeftBracket => {
                state.expect(LeftBracket).ok();
                self.skip_whitespace(state);
                let inner = PrattParser::parse(state, 0, self);
                state.push_child(inner);
                self.skip_whitespace(state);
                state.expect(RightBracket).ok();
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::MemberExpr)
            }
            LeftParen => {
                state.expect(LeftParen).ok();
                self.skip_whitespace(state);
                if !state.at(RightParen) {
                    loop {
                        let before = state.checkpoint().0;
                        let arg = PrattParser::parse(state, 0, self);
                        state.push_child(arg);
                        if state.checkpoint().0 == before {
                            state.finish_at(state.checkpoint(), crate::parser::element_type::VueElementType::Error);
                            break;
                        }
                        self.skip_whitespace(state);
                        if state.eat(Comma) {
                            self.skip_whitespace(state);
                        }
                        else {
                            break;
                        }
                    }
                }
                state.expect(RightParen).ok();
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::CallExpr)
            }
            Question => {
                state.expect(Question).ok();
                let next_prec = match assoc {
                    Associativity::Left => prec + 1,
                    Associativity::Right => prec,
                    Associativity::None => prec + 1,
                };
                self.skip_whitespace(state);
                let then_expr = PrattParser::parse(state, 0, self);
                state.push_child(then_expr);
                self.skip_whitespace(state);
                state.expect(Colon).ok();
                self.skip_whitespace(state);
                let else_expr = PrattParser::parse(state, next_prec, self);
                state.push_child(else_expr);
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::ConditionalExpr)
            }
            _ => {
                state.expect(kind).ok();
                let next_prec = match assoc {
                    Associativity::Left => prec + 1,
                    Associativity::Right => prec,
                    Associativity::None => prec + 1,
                };
                let right = {
                    self.skip_whitespace(state);
                    PrattParser::parse(state, next_prec, self)
                };
                state.push_child(right);
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::BinaryExpr)
            }
        };

        state.sink.restore(op_cp.1);

        state.sink.restore(op_cp.1);
        Some(node)
    }
}
