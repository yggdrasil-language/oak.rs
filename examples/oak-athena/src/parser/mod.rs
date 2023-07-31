#![doc = include_str!("readme.md")]

/// Element kinds.
pub mod element_type;

use crate::{
    language::AthenaLanguage,
    lexer::{AthenaLexer, token_type::AthenaTokenType},
    parser::element_type::AthenaElementType,
};
use oak_core::{
    parser::{
        OperatorInfo, ParseCache, ParseOutput, Parser, ParserState, Pratt, PrattParser, binary, parse_with_lexer,
        unary,
    },
    source::{Source, TextEdit},
    tree::GreenNode,
};

type State<'a, S> = ParserState<'a, AthenaLanguage, S>;

/// Pratt parser for athena.
#[derive(Debug, Clone)]
pub struct AthenaParser<'config> {
    /// Language configuration.
    config: &'config AthenaLanguage,
}

impl<'config> AthenaParser<'config> {
    /// Creates a new parser.
    pub fn new(config: &'config AthenaLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<AthenaLanguage> for AthenaParser<'config> {
    fn parse<'a, S: Source + ?Sized>(
        &self,
        text: &'a S,
        edits: &[TextEdit],
        cache: &'a mut impl ParseCache<AthenaLanguage>,
    ) -> ParseOutput<'a, AthenaLanguage> {
        let lexer = AthenaLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            while state.not_at_end() {
                self.parse_expression(state);
            }
            Ok(state.finish_at(checkpoint, AthenaElementType::Root))
        })
    }
}

impl<'config> AthenaParser<'config> {
    fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        self.parse_pratt(state, 0);
    }

    fn parse_pratt<'a, S: Source + ?Sized>(
        &self,
        state: &mut State<'a, S>,
        min_precedence: u8,
    ) -> &'a GreenNode<'a, AthenaLanguage> {
        PrattParser::new(self.clone()).parse_expr(state, min_precedence)
    }

    fn parse_call_args<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // (
        while state.not_at(AthenaTokenType::RightParen) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(AthenaTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(AthenaTokenType::RightParen) {
            state.bump();
        }
        state.finish_at(checkpoint, AthenaElementType::Arguments);
    }

    fn parse_list_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        state.bump(); // [
        while state.not_at(AthenaTokenType::RightBracket) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(AthenaTokenType::Comma) {
                state.bump();
            } else {
                break;
            }
        }
        if state.at(AthenaTokenType::RightBracket) {
            state.bump();
        }
    }

    fn parse_dict_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        state.bump(); // {
        while state.not_at(AthenaTokenType::RightBrace) && state.not_at_end() {
            let entry = state.checkpoint();
            if state.at(AthenaTokenType::Identifier) || state.at(AthenaTokenType::String) {
                state.bump();
            } else {
                break;
            }
            if state.at(AthenaTokenType::Colon) {
                state.bump();
            }
            self.parse_expression(state);
            state.finish_at(entry, AthenaElementType::DictEntry);
            if state.at(AthenaTokenType::Comma) {
                state.bump();
            } else {
                break;
            }
        }
        if state.at(AthenaTokenType::RightBrace) {
            state.bump();
        }
    }
}

impl<'config> Pratt<AthenaLanguage> for AthenaParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, AthenaLanguage> {
        let checkpoint = state.checkpoint();

        if state.at(AthenaTokenType::Identifier) {
            state.bump();
            if state.at(AthenaTokenType::LeftParen) {
                self.parse_call_args(state);
                return state.finish_at(checkpoint, AthenaElementType::Call);
            }
            return state.finish_at(checkpoint, AthenaElementType::Symbol);
        }

        if state.at(AthenaTokenType::Integer) || state.at(AthenaTokenType::Real) || state.at(AthenaTokenType::String) {
            state.bump();
            return state.finish_at(checkpoint, AthenaElementType::Literal);
        }

        if state.at(AthenaTokenType::LeftParen) {
            state.bump();
            self.parse_expression(state);
            if state.at(AthenaTokenType::RightParen) {
                state.bump();
            }
            return state.finish_at(checkpoint, AthenaElementType::Expression);
        }

        if state.at(AthenaTokenType::LeftBracket) {
            self.parse_list_body(state);
            return state.finish_at(checkpoint, AthenaElementType::List);
        }

        if state.at(AthenaTokenType::LeftBrace) {
            self.parse_dict_body(state);
            return state.finish_at(checkpoint, AthenaElementType::Dict);
        }

        state.bump();
        state.finish_at(checkpoint, AthenaElementType::Error)
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, AthenaLanguage> {
        let kind = match state.peek_kind() {
            Some(k) => k,
            None => return self.primary(state),
        };
        if kind == AthenaTokenType::Minus {
            return unary(state, kind, 150, AthenaElementType::PrefixExpr, |s, p| self.parse_pratt(s, p));
        }
        self.primary(state)
    }

    fn infix<'a, S: Source + ?Sized>(
        &self,
        state: &mut State<'a, S>,
        left: &'a GreenNode<'a, AthenaLanguage>,
        min_precedence: u8,
    ) -> Option<&'a GreenNode<'a, AthenaLanguage>> {
        let kind = state.peek_kind()?;
        let info = match kind {
            AthenaTokenType::Plus | AthenaTokenType::Minus => Some(OperatorInfo::left(80)),
            AthenaTokenType::Times | AthenaTokenType::Divide => Some(OperatorInfo::left(90)),
            AthenaTokenType::Power => Some(OperatorInfo::right(120)),
            _ => None,
        }?;
        if info.precedence < min_precedence {
            return None;
        }
        Some(binary(
            state,
            left,
            kind,
            info.precedence,
            info.associativity,
            AthenaElementType::BinaryExpr,
            |s, p| self.parse_pratt(s, p),
        ))
    }
}
