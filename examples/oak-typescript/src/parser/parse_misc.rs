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
        use crate::{lexer::token_type::TypeScriptTokenType::*, parser::element_type::TypeScriptElementType};

        let cp = state.checkpoint();
        match self.peek_kind(state) {
            Some(LeftParen) => {
                state.bump();
                while state.not_at_end() && !self.at(state, RightParen) {
                    let parameter_cp = state.checkpoint();
                    self.skip_trivia(state);
                    if self.at(state, DotDotDot) {
                        state.bump();
                    }
                    if self.at(state, IdentifierName) || self.at(state, StringLiteral) {
                        state.bump();
                    }
                    self.eat(state, Question);
                    if self.eat(state, Colon) {
                        self.parse_type_annotation(state)?;
                    }
                    state.finish_at(parameter_cp, TypeScriptElementType::Parameter);
                    self.eat(state, Comma);
                    if state.checkpoint().0 == parameter_cp.0 && state.not_at_end() {
                        state.bump();
                    }
                }
                self.expect(state, RightParen).ok();
                if self.eat(state, Arrow) {
                    self.parse_type_annotation(state)?;
                    state.finish_at(cp, TypeScriptElementType::FunctionType);
                }
                else {
                    state.finish_at(cp, TypeScriptElementType::TupleType);
                }
            }
            Some(LeftBrace) => {
                state.bump();
                while state.not_at_end() && !self.at(state, RightBrace) {
                    let member_cp = state.checkpoint();
                    self.skip_trivia(state);
                    if self.at(state, IdentifierName) || self.at(state, StringLiteral) || self.at(state, NumericLiteral) {
                        state.bump();
                        self.eat(state, Question);
                        if self.eat(state, Colon) {
                            self.parse_type_annotation(state)?;
                        }
                        self.eat(state, Comma);
                        self.eat(state, Semicolon);
                        state.finish_at(member_cp, TypeScriptElementType::PropertySignature);
                    }
                    else if state.not_at_end() {
                        state.bump();
                    }
                }
                self.expect(state, RightBrace).ok();
                state.finish_at(cp, TypeScriptElementType::TypeLiteral);
            }
            Some(PredefinedType | Any | Boolean | Never | Number | Object | String | Symbol | Undefined | Unknown | Void) => {
                state.bump();
                state.finish_at(cp, TypeScriptElementType::PredefinedType);
                if self.eat(state, Pipe) {
                    while state.not_at_end() {
                        self.parse_type_annotation(state)?;
                        if !self.eat(state, Pipe) {
                            break;
                        }
                    }
                    state.finish_at(cp, TypeScriptElementType::UnionType);
                }
            }
            Some(StringLiteral | NumericLiteral | True | False) => {
                state.bump();
                state.finish_at(cp, TypeScriptElementType::LiteralType);
            }
            Some(IdentifierName | Typeof | Keyof) => {
                state.bump();
                state.finish_at(cp, TypeScriptElementType::TypeReference);
                if self.eat(state, Pipe) {
                    while state.not_at_end() {
                        self.parse_type_annotation(state)?;
                        if !self.eat(state, Pipe) {
                            break;
                        }
                    }
                    state.finish_at(cp, TypeScriptElementType::UnionType);
                }
            }
            _ => {
                if state.not_at_end() {
                    state.bump();
                    state.finish_at(cp, TypeScriptElementType::TypeReference);
                }
            }
        }
        state.finish_at(cp, TypeScriptElementType::TypeAnnotation);
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
