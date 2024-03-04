use crate::{
    lexer::token_type::TypeScriptTokenType,
    parser::{State, TypeScriptParser},
};
use oak_core::{
    GreenNode,
    parser::pratt::{Associativity, PrattParser, binary},
    source::Source,
};

impl<'config> TypeScriptParser<'config> {
    pub(crate) fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, crate::language::TypeScriptLanguage> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let kind = self.peek_kind(state);
        let cp = state.checkpoint();
        match kind {
            Some(Function) => {
                self.parse_function_declaration_content(state).ok();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::FunctionExpression)
            }
            Some(Import) => {
                state.bump();
                if self.eat(state, Dot) {
                    self.expect(state, IdentifierName).ok();
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::MemberExpression)
                }
                else {
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::IdentifierName)
                }
            }
            Some(IdentifierName) | Some(Undefined) => {
                if self.at(state, IdentifierName) {
                    state.bump();
                }
                else {
                    state.bump();
                }
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::IdentifierName)
            }
            // Keywords that are valid primary expressions (not Error leaves).
            Some(This) => {
                self.expect(state, This).ok();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::This)
            }
            Some(Super) => {
                self.expect(state, Super).ok();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::Super)
            }
            Some(NumericLiteral) | Some(StringLiteral) | Some(BigIntLiteral) | Some(TemplateString) | Some(True) | Some(False) | Some(Null) | Some(RegexLiteral) => {
                let kind = state.peek_kind().unwrap();
                state.bump();
                let elem = match kind {
                    NumericLiteral => crate::parser::element_type::TypeScriptElementType::NumericLiteral,
                    StringLiteral => crate::parser::element_type::TypeScriptElementType::StringLiteral,
                    BigIntLiteral => crate::parser::element_type::TypeScriptElementType::BigIntLiteral,
                    TemplateString => crate::parser::element_type::TypeScriptElementType::TemplateString,
                    True | False => crate::parser::element_type::TypeScriptElementType::BooleanLiteral,
                    Null => crate::parser::element_type::TypeScriptElementType::Null,
                    RegexLiteral => crate::parser::element_type::TypeScriptElementType::RegexLiteral,
                    _ => crate::parser::element_type::TypeScriptElementType::Error,
                };
                state.finish_at(cp, elem)
            }
            Some(LeftParen) => {
                state.bump();
                if !self.at(state, RightParen) {
                    let inner = PrattParser::parse(state, 0, self);
                    state.push_child(inner);
                    if self.eat(state, Colon) {
                        self.parse_type_annotation(state).ok();
                    }
                    while self.eat(state, Comma) {
                        let param = PrattParser::parse(state, 0, self);
                        state.push_child(param);
                        if self.eat(state, Colon) {
                            self.parse_type_annotation(state).ok();
                        }
                    }
                }
                self.expect(state, RightParen).ok();
                if self.eat(state, Arrow) {
                    self.parse_statement(state).ok();
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ArrowFunction)
                }
                else if self.at(state, Colon) {
                    let colon_cp = state.checkpoint();
                    state.bump();
                    self.parse_return_type_annotation(state).ok();
                    if self.eat(state, Arrow) {
                        self.parse_statement(state).ok();
                        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ArrowFunction)
                    }
                    else {
                        state.restore(colon_cp);
                        state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::OpenParen)
                    }
                }
                else {
                    state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::OpenParen)
                }
            }
            Some(Less) if self.config.jsx => self.parse_jsx_element(state),
            Some(LeftBracket) => {
                state.bump();
                while state.not_at_end() && !self.at(state, RightBracket) {
                    let before = state.checkpoint();
                    PrattParser::parse(state, 0, self);
                    self.eat(state, Comma);
                    if state.checkpoint().0 == before.0 && state.not_at_end() {
                        state.bump();
                    }
                }
                self.expect(state, RightBracket).ok();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ArrayExpression)
            }
            Some(LeftBrace) => {
                state.bump();
                while state.not_at_end() && !self.at(state, RightBrace) {
                    let pcp = state.checkpoint();
                    if self.eat(state, DotDotDot) {
                        PrattParser::parse(state, 0, self);
                        state.finish_at(pcp, crate::parser::element_type::TypeScriptElementType::SpreadElement);
                    }
                    else {
                        match self.peek_kind(state) {
                            Some(IdentifierName) => {
                                self.expect(state, IdentifierName).ok();
                            }
                            Some(StringLiteral) | Some(NumericLiteral) => {
                                state.bump();
                            }
                            Some(LeftBracket) => {
                                state.bump();
                                PrattParser::parse(state, 0, self);
                                self.expect(state, RightBracket).ok();
                            }
                            _ => {
                                state.bump();
                                state.finish_at(pcp, crate::parser::element_type::TypeScriptElementType::Error);
                                continue;
                            }
                        }
                        if self.eat(state, Colon) {
                            PrattParser::parse(state, 0, self);
                            state.finish_at(pcp, crate::parser::element_type::TypeScriptElementType::PropertyAssignment);
                        }
                        else {
                            state.finish_at(pcp, crate::parser::element_type::TypeScriptElementType::ShorthandPropertyAssignment);
                        }
                    }
                    self.eat(state, Comma);
                    if state.checkpoint().0 == pcp.0 && state.not_at_end() {
                        state.bump();
                    }
                }
                self.expect(state, RightBrace).ok();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ObjectExpression)
            }
            _ => {
                state.bump();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::Error)
            }
        }
    }

    pub(crate) fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, crate::language::TypeScriptLanguage> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let kind = self.peek_kind(state);
        let cp = state.checkpoint();
        match kind {
            Some(PlusPlus) | Some(MinusMinus) => {
                state.bump();
                PrattParser::parse(state, 15, self);
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::UpdateExpression.into())
            }
            Some(Plus) | Some(Minus) | Some(Exclamation) | Some(Tilde) | Some(Typeof) | Some(Void) | Some(Delete) | Some(Await) => {
                state.bump();
                PrattParser::parse(state, 15, self); // High precedence for prefix
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::UnaryExpression.into())
            }
            Some(Import) => {
                state.bump();
                self.expect(state, LeftParen).ok();
                PrattParser::parse(state, 0, self);
                self.expect(state, RightParen).ok();
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::CallExpression.into()) // We can use CallExpression kind for now, or a new one
            }
            Some(New) => {
                state.bump();
                PrattParser::parse(state, 17, self); // Higher precedence than call
                if self.eat(state, LeftParen) {
                    while state.not_at_end() && !self.at(state, RightParen) {
                        let acp = state.checkpoint();
                        PrattParser::parse(state, 0, self);
                        state.finish_at(acp, crate::parser::element_type::TypeScriptElementType::CallArgument);
                        self.eat(state, Comma);
                    }
                    self.expect(state, RightParen).ok();
                }
                state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::NewExpression.into())
            }
            _ => self.primary(state),
        }
    }

    pub(crate) fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, crate::language::TypeScriptLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, crate::language::TypeScriptLanguage>> {
        use crate::lexer::token_type::TypeScriptTokenType::*;
        let kind = self.peek_kind(state)?;

        let (prec, assoc) = match kind {
            Equal
            | PlusEqual
            | MinusEqual
            | StarEqual
            | SlashEqual
            | PercentEqual
            | StarStarEqual
            | LeftShiftEqual
            | RightShiftEqual
            | UnsignedRightShiftEqual
            | AmpersandEqual
            | PipeEqual
            | CaretEqual
            | AmpersandAmpersandEqual
            | PipePipeEqual
            | QuestionQuestionEqual => (1, Associativity::Right),
            Question => (2, Associativity::Right),
            PipePipe => (3, Associativity::Left),
            QuestionQuestion => (3, Associativity::Left),
            AmpersandAmpersand => (4, Associativity::Left),
            Pipe => (5, Associativity::Left),
            Caret => (6, Associativity::Left),
            Ampersand => (7, Associativity::Left),
            EqualEqual | NotEqual | EqualEqualEqual | NotEqualEqual => (8, Associativity::Left),
            Less | Greater | LessEqual | GreaterEqual | Instanceof | In => (9, Associativity::Left),
            LeftShift | RightShift | UnsignedRightShift => (10, Associativity::Left),
            Plus | Minus => (11, Associativity::Left),
            Star | Slash | Percent => (12, Associativity::Left),
            StarStar => (13, Associativity::Right),
            As => (14, Associativity::Left),
            Arrow => (15, Associativity::Right),
            LeftParen | Dot | LeftBracket | QuestionDot => (16, Associativity::Left),
            // Postfix ++ / -- bind tighter than call/member so `this.count++` is UpdateExpression.
            PlusPlus | MinusMinus => (17, Associativity::Left),
            _ => return None,
        };

        if prec < min_precedence {
            return None;
        }

        match kind {
            PlusPlus | MinusMinus => {
                let cp = state.checkpoint_before(left);
                state.bump();
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::UpdateExpression.into()))
            }
            LeftParen => {
                let cp = state.checkpoint_before(left);
                self.expect(state, LeftParen).ok();
                while state.not_at_end() && !self.at(state, RightParen) {
                    self.skip_trivia(state);
                    let acp = state.checkpoint();
                    if self.at(state, Import) {
                        state.bump();
                        self.eat(state, Dot);
                        self.expect(state, IdentifierName).ok();
                        if self.eat(state, Dot) {
                            self.expect(state, IdentifierName).ok();
                        }
                    }
                    else {
                        PrattParser::parse(state, 0, self);
                    }
                    state.finish_at(acp, crate::parser::element_type::TypeScriptElementType::CallArgument);
                    if state.checkpoint().0 == acp.0 {
                        state.bump();
                        break;
                    }
                    self.eat(state, Comma);
                }
                self.expect(state, RightParen).ok();
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::CallExpression.into()))
            }
            Dot | QuestionDot => {
                let cp = state.checkpoint_before(left);
                let op = if kind == Dot { Dot } else { QuestionDot };
                self.expect(state, op).ok();
                if kind == QuestionDot && self.at(state, LeftParen) {
                    self.expect(state, LeftParen).ok();
                    while state.not_at_end() && !self.at(state, RightParen) {
                        let acp = state.checkpoint();
                        PrattParser::parse(state, 0, self);
                        state.finish_at(acp, crate::parser::element_type::TypeScriptElementType::CallArgument);
                        if !self.eat(state, Comma) {
                            break;
                        }
                    }
                    self.expect(state, RightParen).ok();
                    return Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::CallExpression.into()));
                }
                if self.at(state, IdentifierName) || state.peek_text().is_some_and(|text| TypeScriptTokenType::from_keyword(&text).is_some()) {
                    state.bump();
                }
                else {
                    self.expect(state, IdentifierName).ok();
                }
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::MemberExpression.into()))
            }
            LeftBracket => {
                let cp = state.checkpoint_before(left);
                self.expect(state, LeftBracket).ok();
                PrattParser::parse(state, 0, self);
                self.expect(state, RightBracket).ok();
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::MemberExpression.into()))
            }
            As => {
                let cp = state.checkpoint_before(left);
                self.expect(state, As).ok();
                // Simple type handling: skip next identifier or basic type
                self.skip_trivia(state);
                if state.at(IdentifierName.into()) {
                    self.expect(state, IdentifierName).ok();
                }
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::AsExpression.into()))
            }
            Arrow => {
                let cp = state.checkpoint_before(left);
                self.expect(state, Arrow).ok();
                self.parse_statement(state).ok();
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ArrowFunction.into()))
            }
            Question => {
                let cp = state.checkpoint_before(left);
                self.expect(state, Question).ok();
                PrattParser::parse(state, 0, self);
                self.expect(state, Colon).ok();
                PrattParser::parse(state, 0, self);
                Some(state.finish_at(cp, crate::parser::element_type::TypeScriptElementType::ConditionalExpression.into()))
            }
            _ => Some(binary(state, left, kind, prec, assoc, crate::parser::element_type::TypeScriptElementType::BinaryExpression, |s, p| PrattParser::parse(s, p, self))),
        }
    }
}
