use crate::{
    lexer::token_type::TypeScriptTokenType,
    parser::{State, TypeScriptParser},
};
use oak_core::{OakError, parser::pratt::PrattParser, source::Source};

impl<'config> TypeScriptParser<'config> {
    pub(crate) fn parse_class_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let cp = state.checkpoint();
        self.parse_class_declaration_content(state)?;
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ClassDeclaration);
        Ok(())
    }

    pub(crate) fn parse_class_declaration_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        self.eat(state, Abstract);
        self.expect(state, Class).ok();
        self.expect(state, IdentifierName).ok();
        if self.eat(state, Extends) {
            self.expect(state, IdentifierName).ok();
        }
        if self.eat(state, Implements) {
            while state.not_at_end() && !self.at(state, LeftBrace) {
                self.expect(state, IdentifierName).ok();
                if !self.eat(state, Comma) {
                    break;
                }
            }
        }
        self.parse_class_body(state)?;
        Ok(())
    }

    pub(crate) fn parse_class_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let cp = state.checkpoint();
        self.expect(state, LeftBrace).ok();
        while state.not_at_end() && !self.at(state, RightBrace) {
            self.skip_trivia(state);
            let mcp = state.checkpoint();

            // Handle member decorators
            while self.at(state, At) {
                self.parse_decorator(state)?;
            }

            // Parse modifiers
            while state.not_at_end() {
                let kind = self.peek_kind(state);
                match kind {
                    Some(TypeScriptTokenType::Static)
                    | Some(TypeScriptTokenType::Public)
                    | Some(TypeScriptTokenType::Private)
                    | Some(TypeScriptTokenType::Protected)
                    | Some(TypeScriptTokenType::Readonly)
                    | Some(TypeScriptTokenType::Abstract)
                    | Some(TypeScriptTokenType::Async)
                    | Some(TypeScriptTokenType::Override) => state.bump(),
                    _ => break,
                }
            }

            // Handle get/set
            if self.at(state, Get) || self.at(state, Set) {
                state.bump();
            }

            let member_kind = self.peek_kind(state);
            let keyword_name = state.peek_text().is_some_and(|text| TypeScriptTokenType::from_keyword(&text).is_some());
            if member_kind == Some(IdentifierName) || (keyword_name && member_kind != Some(Constructor)) {
                let name_cp = state.checkpoint();
                state.bump();
                state.finish_at(name_cp, crate::parser::element_type::TypeScriptElementType::IdentifierName);
                if self.at(state, LeftParen) {
                    self.parse_parameters(state)?;
                    self.skip_trivia(state);
                    if self.eat(state, Colon) {
                        // Skip return type
                        while state.not_at_end() && !self.at(state, LeftBrace) && !self.at(state, Semicolon) && !self.at(state, RightBrace) {
                            state.bump();
                        }
                    }
                    if self.at(state, LeftBrace) {
                        self.parse_block(state)?;
                    }
                    else {
                        self.eat(state, Semicolon);
                    }
                    state.finish_at(mcp, crate::parser::element_type::TypeScriptElementType::MethodDeclaration);
                }
                else {
                    if self.eat(state, Colon) {
                        self.skip_type_until_member_boundary(state);
                    }
                    if self.eat(state, Equal) {
                        PrattParser::parse(state, 0, self);
                    }
                    self.eat(state, Semicolon);
                    state.finish_at(mcp, crate::parser::element_type::TypeScriptElementType::PropertyDeclaration);
                }
            }
            else if self.at(state, Constructor) {
                state.bump();
                self.parse_parameters(state)?;
                self.skip_trivia(state);
                if self.eat(state, Colon) {
                    self.expect(state, IdentifierName).ok();
                }
                self.parse_block(state)?;
                state.finish_at(mcp, crate::parser::element_type::TypeScriptElementType::ConstructorDeclaration);
            }
            else if self.at(state, Semicolon) {
                state.bump(); // Skip extra semicolons in class body
            }
            else if !self.at(state, RightBrace) && state.not_at_end() {
                // If we encounter something unexpected, try to skip until next member or end of class
                state.bump();
            }
            else {
                break;
            }

            if state.checkpoint().0 == mcp.0 && state.not_at_end() {
                state.bump();
            }
        }
        self.expect(state, RightBrace).ok();
        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ClassBody);
        Ok(())
    }

    fn skip_type_until_member_boundary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        use crate::lexer::token_type::TypeScriptTokenType::*;

        let mut parens = 0usize;
        let mut brackets = 0usize;
        let mut braces = 0usize;
        let mut angles = 0usize;
        let mut previous = None;
        while state.not_at_end() {
            let Some(kind) = self.peek_kind(state)
            else {
                break;
            };
            match kind {
                Equal if previous != Some(Greater) && parens == 0 && brackets == 0 && braces == 0 && angles == 0 => break,
                Semicolon if parens == 0 && brackets == 0 && braces == 0 && angles == 0 => break,
                RightBrace if parens == 0 && brackets == 0 && braces == 0 && angles == 0 => break,
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
            previous = Some(kind);
            state.bump();
        }
    }
}
