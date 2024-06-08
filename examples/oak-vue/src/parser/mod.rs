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
                let before = state.checkpoint().0;
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
                // Every root-level token must make progress: a parser branch that returns
                // without consuming would otherwise spin forever on the same token.
                if state.checkpoint().0 == before && state.not_at_end() {
                    state.bump();
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
        state.finish_at(tag_cp, crate::parser::element_type::VueElementType::Tag);

        if tag_name == "script" {
            self.parse_program(state);
        }
        else if recursive {
            // Only the matching `</template>` closes the block. Any other close tag is an
            // ordinary invalid end tag that Vue reports and keeps parsing, so it must be
            // consumed as a node instead of being mistaken for the block close.
            while state.not_at_end() {
                if state.at(VueTokenType::LtSlash) && Self::close_tag_matches(state, tag_name) {
                    break;
                }
                let before = state.checkpoint().0;
                self.parse_node(state);
                if state.checkpoint().0 == before && state.not_at_end() {
                    state.bump();
                }
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
            state.finish_at(close_cp, crate::parser::element_type::VueElementType::CloseTag);
        }

        let block_kind = match tag_name {
            "template" => crate::parser::element_type::VueElementType::TemplateElement,
            "script" => crate::parser::element_type::VueElementType::Program,
            _ => crate::parser::element_type::VueElementType::Element,
        };
        state.finish_at(cp, block_kind);
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
            state.finish_at(cp, crate::parser::element_type::VueElementType::CommentNode);
        }
        else if state.at(VueTokenType::LtSlash) {
            // A closing tag with no open element (`</div>` at root). Vue reports
            // `X_INVALID_END_TAG` and keeps parsing, so consume the tag to make progress.
            let cp = state.checkpoint();
            state.expect(VueTokenType::LtSlash).ok();
            let close_name = match Self::bump_attr_name(state) {
                Some(name) => name,
                None => {
                    state.bump();
                    String::new()
                }
            };
            // Browsers drop a close tag for a void element (`</img>`, `</br>`) and Vue
            // accepts it without a diagnostic, so do not report an invalid end tag there.
            if !is_html_void_tag(&close_name) {
                let _ = state.syntax_error("invalid end tag".to_string());
            }
            state.expect(VueTokenType::Gt).ok();
            state.finish_at(cp, crate::parser::element_type::VueElementType::CloseTag);
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
            state.finish_at(cp, crate::parser::element_type::VueElementType::TextNode);
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
            state.finish_at(cp, crate::parser::element_type::VueElementType::Element);
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
            // Vue's tokenizer rejects `/` inside a start tag (only `/>` closes it).
            if state.at(VueTokenType::Slash) {
                let _ = state.syntax_error("illegal `/` in tags".to_string());
                state.bump();
                continue;
            }
            // A `<` inside a start tag means the tag was never closed: Vue reports
            // `X_UNEXPECTED_EOF_IN_TAG` instead of swallowing the next tag as attributes.
            if state.at(VueTokenType::Lt) || state.at(VueTokenType::LtSlash) {
                let _ = state.syntax_error("unexpected `<` in tag".to_string());
                break;
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

        state.finish_at(tag_cp, crate::parser::element_type::VueElementType::Tag);

        if !is_self_closing && !is_html_void_tag(&tag_name) {
            // Only the matching `</name>` closes this element. Any other close tag is handled
            // by `parse_node`, which reports an invalid end tag or ignores a void element's
            // close tag, matching Vue instead of closing the wrong element.
            while state.not_at_end() {
                if state.at(VueTokenType::LtSlash) && Self::close_tag_matches(state, &tag_name) {
                    break;
                }
                let before = state.checkpoint().0;
                self.parse_node(state);
                if state.checkpoint().0 == before && state.not_at_end() {
                    state.bump();
                }
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
                state.finish_at(close_cp, crate::parser::element_type::VueElementType::CloseTag);
            }
            // Vue reports `X_MISSING_END_TAG` when the element runs into EOF.
            else {
                let _ = state.syntax_error(format!("element is missing end tag: `<{tag_name}>`"));
            }
        }

        state.finish_at(cp, crate::parser::element_type::VueElementType::Element);
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
            directive_name = attribute_name.as_deref().unwrap_or_default().to_string();
        }

        if is_directive && (adjacent_colon || state.at(VueTokenType::LeftBracket)) {
            self.parse_optional_directive_arg(state);
        }

        // Vue keeps `:"x"` / `v-bind:"x"` legal: the quoted token becomes the argument.
        if is_directive && state.at(VueTokenType::StringLiteral) {
            state.bump();
        }
        // Vue reports `X_INVALID_ATTRIBUTE_NAME` when a quote reaches an attribute-name
        // position. Directive names keep Vue's looser tokenizer behaviour (`:"x"` parses),
        // so only plain attributes fail here.
        if !is_directive && state.at(VueTokenType::StringLiteral) {
            let _ = state.syntax_error("attribute name cannot contain quotes".to_string());
            state.bump();
        }
        // A bare `=` cannot start an attribute name (`<div =x>`).
        if !is_directive && attribute_name.is_none() && state.at(VueTokenType::Eq) {
            let _ = state.syntax_error("attribute name cannot start with `=`".to_string());
            state.bump();
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
            else {
                // Vue reads an unquoted attribute value as raw text up to whitespace or the
                // tag terminator. Parsing it as a JavaScript expression here consumes `>` as
                // an operator and swallows the rest of the element (including its close tag).
                Self::bump_unquoted_attr_value(state);
            }
            state.finish_at(val_cp, crate::parser::element_type::VueElementType::AttributeValue);
        }
        // `a==b` / `a>=b`: a compound `=` token cannot start an attribute value in Vue.
        else if state.at(VueTokenType::EqEq) || state.at(VueTokenType::EqEqEq) || state.at(VueTokenType::Arrow) || state.at(VueTokenType::GtEq) || state.at(VueTokenType::LtEq) || state.at(VueTokenType::NotEq) {
            let _ = state.syntax_error("unexpected `=` in attribute".to_string());
        }

        let kind = if is_directive { crate::parser::element_type::VueElementType::Directive } else { crate::parser::element_type::VueElementType::Attribute };
        state.finish_at(cp, kind);

        // Last-resort progress: empty attribute nodes must not be emitted forever.
        if state.checkpoint().0 == cp.0 && state.not_at_end() {
            state.bump();
        }
    }

    /// `true` when the `</name>` at the cursor closes the given block tag.
    fn close_tag_matches<'a, S: Source + ?Sized>(state: &State<'a, S>, tag_name: &str) -> bool {
        let Some(token) = state.peek_at(1)
        else {
            return false;
        };
        state.source.get_text_in(token.span).eq_ignore_ascii_case(tag_name)
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
            state.finish_at(arg_cp, crate::parser::element_type::VueElementType::Identifier);
        }
        else if state.at(VueTokenType::Identifier) || Self::bump_attr_name_is_keyword(state.peek_kind()) {
            let arg_cp = state.checkpoint();
            state.bump();
            state.finish_at(arg_cp, crate::parser::element_type::VueElementType::Identifier);
        }
    }

    fn bump_attr_name_is_keyword(kind: Option<VueTokenType>) -> bool {
        use VueTokenType::*;
        matches!(kind, Some(For | In | Of | As | If | Else | From | Default | Const | Let | Var | Function | Return | Break | Continue | Switch | Try | Throw | True | False | Null | Import | Export | While))
    }

    fn parse_directive_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, directive_name: &str) {
        if !state.at(VueTokenType::StringLiteral) {
            Self::bump_unquoted_attr_value(state);
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

    /// Consume an unquoted attribute value as raw text (Vue `parseAttributeValue`).
    fn bump_unquoted_attr_value<'a, S: Source + ?Sized>(state: &mut State<'a, S>) {
        // `bump` skips trivia, so a `Whitespace` token never shows up as the current
        // token. Detect the end of the value through byte adjacency instead: Vue ends
        // an unquoted value at the first whitespace or the tag terminator.
        // A quoted value is one lexer token; consume it whole.
        if state.at(VueTokenType::StringLiteral) {
            state.bump();
            return;
        }
        let mut previous_end: Option<usize> = None;
        while state.not_at_end() && !state.at(VueTokenType::Gt) && !state.at(VueTokenType::SlashGt) {
            let Some(token) = state.current()
            else {
                break;
            };
            if previous_end.is_some_and(|end| token.span.start != end) {
                break;
            }
            // Vue's tokenizer ends an unquoted value at `"`, `'`, `` ` ``, `<` and `=`, and
            // reports `X_UNEXPECTED_...` for the stray character instead of folding it into
            // the value.
            let stray_backtick = state.at(VueTokenType::Text) && state.peek_text().is_some_and(|text| text.contains('`'));
            if stray_backtick || state.at(VueTokenType::StringLiteral) || state.at(VueTokenType::Eq) || state.at(VueTokenType::Lt) {
                let _ = state.syntax_error("unquoted attribute value cannot contain quotes, backticks, `<` or `=`".to_string());
                break;
            }
            previous_end = Some(token.span.end);
            state.bump();
        }
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
        self.parse_expression(state);

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
        self.parse_expression(state);
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
        if state.at(LeftParen) {
            state.expect(LeftParen).ok();
            if !state.at(RightParen) {
                PrattParser::parse(state, 0, self);
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
                        PrattParser::parse(state, 0, self);
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
        }
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
                self.skip_whitespace(state);
                unary(state, kind.into(), 90, UnaryExpr.into(), |s, p| PrattParser::parse(s, p, self))
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

        // `left` already sits in the sink at its source position, so wrap from the node
        // itself instead of re-pushing a detached copy.
        state.restore(start_cp);
        let op_cp = state.checkpoint_before(left);
        self.skip_whitespace(state);

        let node = match kind {
            Arrow => {
                state.expect(Arrow).ok();
                self.skip_whitespace(state);
                PrattParser::parse(state, 0, self);
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
                PrattParser::parse(state, 0, self);
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
                        PrattParser::parse(state, 0, self);
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
                PrattParser::parse(state, 0, self);
                self.skip_whitespace(state);
                state.expect(Colon).ok();
                self.skip_whitespace(state);
                PrattParser::parse(state, next_prec, self);
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::ConditionalExpr)
            }
            _ => {
                state.expect(kind).ok();
                let next_prec = match assoc {
                    Associativity::Left => prec + 1,
                    Associativity::Right => prec,
                    Associativity::None => prec + 1,
                };
                self.skip_whitespace(state);
                PrattParser::parse(state, next_prec, self);
                state.finish_at(op_cp, crate::parser::element_type::VueElementType::BinaryExpr)
            }
        };

        Some(node)
    }
}
