use crate::parser::{State, TypeScriptParser, element_type::TypeScriptElementType};
use oak_core::{GreenNode, parser::pratt::PrattParser, source::Source};

impl<'config> TypeScriptParser<'config> {
    pub(crate) fn parse_jsx_element<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, crate::language::TypeScriptLanguage> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();

        if self.eat(state, Less) {
            if self.eat(state, Greater) {
                let opening_cp = state.checkpoint();
                state.finish_at(opening_cp, TypeScriptElementType::JsxOpeningFragment);
                self.parse_jsx_children(state);
                let closing_cp = state.checkpoint();
                self.expect(state, Less).ok();
                self.expect(state, Slash).ok();
                self.expect(state, Greater).ok();
                state.finish_at(closing_cp, TypeScriptElementType::JsxClosingFragment);
                return state.finish_at(cp, TypeScriptElementType::JsxFragment);
            }

            let opening_cp = state.checkpoint();
            self.parse_jsx_name(state);
            self.parse_jsx_attributes(state);
            if self.eat(state, Slash) {
                self.expect(state, Greater).ok();
                return state.finish_at(cp, TypeScriptElementType::JsxSelfClosingElement);
            }

            self.expect(state, Greater).ok();
            state.finish_at(opening_cp, TypeScriptElementType::JsxOpeningElement);
            self.parse_jsx_children(state);
            if self.eat(state, Less) {
                self.eat(state, Slash);
                let closing_cp = state.checkpoint();
                self.parse_jsx_name(state);
                self.expect(state, Greater).ok();
                state.finish_at(closing_cp, TypeScriptElementType::JsxClosingElement);
            }
            return state.finish_at(cp, TypeScriptElementType::JsxElement);
        }

        state.finish_at(cp, TypeScriptElementType::Error)
    }

    pub(crate) fn parse_jsx_name<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        self.expect(state, IdentifierName).ok();
        while self.eat(state, Dot) {
            self.expect(state, IdentifierName).ok();
        }
        state.finish_at(cp, TypeScriptElementType::IdentifierName);
    }

    fn parse_jsx_attributes<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let attrs_cp = state.checkpoint();
        let mut has_attrs = false;
        while state.not_at_end() && !self.at(state, Greater) && !self.at(state, Slash) {
            self.parse_jsx_attribute(state);
            has_attrs = true;
        }
        if has_attrs {
            state.finish_at(attrs_cp, TypeScriptElementType::JsxAttributes);
        }
    }

    pub(crate) fn parse_jsx_attribute<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        if self.eat(state, LeftBrace) {
            self.expect(state, DotDotDot).ok();
            PrattParser::parse(state, 0, self);
            self.expect(state, RightBrace).ok();
            state.finish_at(cp, TypeScriptElementType::JsxSpreadAttribute);
        }
        else {
            let name_cp = state.checkpoint();
            self.expect(state, IdentifierName).ok();
            state.finish_at(name_cp, TypeScriptElementType::IdentifierName);
            if self.eat(state, Equal) {
                if self.at(state, StringLiteral) {
                    let lit_cp = state.checkpoint();
                    state.bump();
                    state.finish_at(lit_cp, TypeScriptElementType::StringLiteral);
                }
                else if self.eat(state, LeftBrace) {
                    let expr_cp = state.checkpoint();
                    PrattParser::parse(state, 0, self);
                    self.expect(state, RightBrace).ok();
                    state.finish_at(expr_cp, TypeScriptElementType::JsxExpressionContainer);
                }
            }
            state.finish_at(cp, TypeScriptElementType::JsxAttribute);
        }
    }

    pub(crate) fn parse_jsx_children<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        while state.not_at_end() && !self.at_jsx_closing_tag(state) {
            if self.at(state, Less) {
                self.parse_jsx_element(state);
                continue;
            }
            let text_cp = state.checkpoint();
            while state.not_at_end() && !self.at(state, Less) {
                state.bump();
            }
            if text_cp.0 != state.checkpoint().0 {
                state.finish_at(text_cp, TypeScriptElementType::JsxText);
            }
        }
    }

    fn at_jsx_closing_tag<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        if !self.at(state, Less) {
            return false;
        }
        let saved = state.checkpoint();
        state.bump();
        let is_close = self.at(state, Slash);
        state.restore(saved);
        is_close
    }
}
