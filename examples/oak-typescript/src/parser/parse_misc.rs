use crate::{
    lexer::TypeScriptTokenType,
    parser::{State, TypeScriptParser},
};
use oak_core::{OakError, TokenType, source::Source};

impl<'config> TypeScriptParser<'config> {
    pub(crate) fn peek_kind<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Option<TypeScriptTokenType> {
        self.skip_trivia(state);
        state.peek_kind().map(|k| k.try_into().unwrap())
    }

    pub(crate) fn skip_trivia<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.not_at_end() && state.current().map(|t| t.kind.is_ignored()).unwrap_or(false) {
            state.bump();
        }
    }

    pub(crate) fn expect<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, kind: TypeScriptTokenType) -> Result<(), OakError> {
        self.skip_trivia(state);
        state.expect(kind)
    }

    pub(crate) fn eat<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, kind: TypeScriptTokenType) -> bool {
        self.skip_trivia(state);
        state.eat(kind)
    }

    pub(crate) fn at<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, kind: TypeScriptTokenType) -> bool {
        self.skip_trivia(state);
        state.at(kind.into())
    }

    pub(crate) fn parse_parameters<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        self.expect(state, LeftParen).ok();
        while state.not_at_end() && !self.at(state, RightParen) {
            self.skip_trivia(state);
            let cp = state.checkpoint();

            // Handle parameter decorators
            while self.at(state, At) {
                self.parse_decorator(state)?;
            }

            if self.at(state, IdentifierName) {
                self.expect(state, IdentifierName).ok();
                // Skip type annotation
                if self.eat(state, Colon) {
                    while state.not_at_end() && !self.at(state, Comma) && !self.at(state, RightParen) {
                        self.skip_trivia(state);
                        if state.not_at_end() && !self.at(state, Comma) && !self.at(state, RightParen) { state.bump() } else { break }
                    }
                }
            }
            else {
                state.bump();
            }
            state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::Parameter);
            self.eat(state, Comma);
        }
        self.expect(state, RightParen).ok();
        Ok(())
    }

    pub(crate) fn parse_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            let before = state.checkpoint();
            self.parse_statement(state)?;
            if state.checkpoint().0 == before.0 && state.not_at_end() {
                state.bump();
            }
        }
        self.expect(state, RightBrace).ok();
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::BlockStatement);
        Ok(())
    }
}
