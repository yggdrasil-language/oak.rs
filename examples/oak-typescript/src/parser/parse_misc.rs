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
                    self.parse_type_annotation(state)?;
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

    pub(crate) fn parse_type_annotation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        self.parse_type_annotation_with_arrow(state, false)
    }

    pub(crate) fn parse_return_type_annotation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        self.parse_type_annotation_with_arrow(state, true)
    }

    fn parse_type_annotation_with_arrow<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, stop_at_arrow: bool) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        use crate::parser::element_type::TypeScriptElementType;

        let cp = state.checkpoint();
        let mut parens = 0usize;
        let mut brackets = 0usize;
        let mut braces = 0usize;
        let mut angles = 0usize;
        let mut consumed = false;

        while state.not_at_end() {
            let Some(kind) = self.peek_kind(state) else { break };
            if consumed && parens == 0 && brackets == 0 && braces == 0 && angles == 0 && (matches!(kind, Comma | Semicolon | RightParen | RightBrace | Equal) || (stop_at_arrow && matches!(kind, Arrow))) {
                break;
            }
            match kind {
                LeftParen => parens += 1,
                RightParen if parens > 0 => parens -= 1,
                LeftBracket => brackets += 1,
                RightBracket if brackets > 0 => brackets -= 1,
                LeftBrace => braces += 1,
                RightBrace if braces > 0 => braces -= 1,
                Less => angles += 1,
                Greater if angles > 0 => angles -= 1,
                _ => {}
            }
            state.bump();
            consumed = true;
        }

        if consumed {
            state.finish_at(cp, TypeScriptElementType::TypeAnnotation);
        }
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
