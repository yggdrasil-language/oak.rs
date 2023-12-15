pub mod element_type;

use crate::{
    language::MapleLanguage,
    lexer::{MapleLexer, token_type::MapleTokenType},
    parser::element_type::MapleElementType,
};
use oak_core::{
    parser::{OperatorInfo, ParseCache, ParseOutput, Parser, ParserState, Pratt, PrattParser, binary, parse_with_lexer, unary},
    source::{Source, TextEdit},
    tree::{GreenNode, GreenTree},
};
type State<'a, SourceType> = ParserState<'a, MapleLanguage, SourceType>;

/// Parses the supported Maple expression subset into a lossless syntax tree.
#[derive(Debug, Clone)]
pub struct MapleParser<'config> {
    config: &'config MapleLanguage,
}
impl<'config> MapleParser<'config> {
    /// Creates a parser using the supplied language configuration.
    pub fn new(config: &'config MapleLanguage) -> Self {
        Self { config }
    }

    fn parse_pratt<'a, SourceType: Source + ?Sized>(&self, state: &mut State<'a, SourceType>, minimum: u8) -> &'a GreenNode<'a, MapleLanguage> {
        PrattParser::new(self.clone()).parse_expr(state, minimum)
    }

    fn sequence<'a, SourceType: Source + ?Sized>(&self, state: &mut State<'a, SourceType>, closing: MapleTokenType) {
        state.bump();
        if !state.at(closing) {
            loop {
                self.parse_pratt(state, 0);
                if !state.eat(MapleTokenType::Comma) {
                    break;
                }
                if state.at(closing) {
                    state.record_trailing_comma_not_allowed();
                    break;
                }
                if state.at(MapleTokenType::Eof) || !state.not_at_end() {
                    break;
                }
            }
        }
        let _ = state.expect(closing);
    }
}
impl<'config> Parser<MapleLanguage> for MapleParser<'config> {
    fn parse<'a, SourceType: Source + ?Sized>(&self, text: &'a SourceType, edits: &[TextEdit], cache: &'a mut impl ParseCache<MapleLanguage>) -> ParseOutput<'a, MapleLanguage> {
        let lexer = MapleLexer::new(self.config);
        cache.prepare_generation();
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            state.skip_trivia();
            while state.not_at_end() && !state.at(MapleTokenType::Eof) {
                self.parse_pratt(state, 0);
                if !state.eat(MapleTokenType::Semicolon) && state.not_at_end() && !state.at(MapleTokenType::Eof) {
                    state.syntax_error("Expected semicolon between expressions");
                    while state.not_at_end() && !state.at(MapleTokenType::Eof) && !state.at(MapleTokenType::Semicolon) {
                        state.bump();
                    }
                    state.eat(MapleTokenType::Semicolon);
                }
            }
            Ok(state.finish_at(checkpoint, MapleElementType::Root))
        })
    }
}
impl<'config> Pratt<MapleLanguage> for MapleParser<'config> {
    fn primary<'a, SourceType: Source + ?Sized>(&self, state: &mut State<'a, SourceType>) -> &'a GreenNode<'a, MapleLanguage> {
        let checkpoint = state.checkpoint();
        if state.at(MapleTokenType::Identifier) {
            state.bump();
            if state.at(MapleTokenType::LeftParen) {
                let arguments = state.checkpoint();
                self.sequence(state, MapleTokenType::RightParen);
                state.finish_at(arguments, MapleElementType::Arguments);
                return state.finish_at(checkpoint, MapleElementType::Call);
            }
            return state.finish_at(checkpoint, MapleElementType::Symbol);
        }
        if state.at(MapleTokenType::Integer) || state.at(MapleTokenType::Float) {
            state.bump();
            return state.finish_at(checkpoint, MapleElementType::Literal);
        }
        if state.eat(MapleTokenType::LeftParen) {
            self.parse_pratt(state, 0);
            let _ = state.expect(MapleTokenType::RightParen);
            return state.finish_at(checkpoint, MapleElementType::Expression);
        }
        if state.at(MapleTokenType::LeftBracket) {
            self.sequence(state, MapleTokenType::RightBracket);
            return state.finish_at(checkpoint, MapleElementType::List);
        }
        state.syntax_error("Expected Maple expression");
        if !matches!(state.peek_kind(), None | Some(MapleTokenType::Eof | MapleTokenType::RightParen | MapleTokenType::RightBracket | MapleTokenType::Comma | MapleTokenType::Semicolon)) {
            state.bump();
        }
        state.finish_at(checkpoint, MapleElementType::Error)
    }
    fn prefix<'a, SourceType: Source + ?Sized>(&self, state: &mut State<'a, SourceType>) -> &'a GreenNode<'a, MapleLanguage> {
        if matches!(state.peek_kind(), Some(MapleTokenType::Minus | MapleTokenType::Plus)) {
            let operator = state.peek_kind().unwrap();
            return unary(state, operator, 81, MapleElementType::PrefixExpr, |state, minimum| self.parse_pratt(state, minimum));
        }
        self.primary(state)
    }
    fn infix<'a, SourceType: Source + ?Sized>(&self, state: &mut State<'a, SourceType>, left: &'a GreenNode<'a, MapleLanguage>, minimum: u8) -> Option<&'a GreenNode<'a, MapleLanguage>> {
        let operator = state.peek_kind()?;
        let info = match operator {
            MapleTokenType::Plus | MapleTokenType::Minus => OperatorInfo::left(80),
            MapleTokenType::Times | MapleTokenType::Divide => OperatorInfo::left(90),
            MapleTokenType::Power => OperatorInfo::none(120),
            _ => return None,
        };
        if info.precedence < minimum {
            return None;
        }
        if operator == MapleTokenType::Power && left.kind == MapleElementType::BinaryExpr && left.children.iter().any(|child| matches!(child, GreenTree::Leaf(leaf) if leaf.kind == MapleTokenType::Power)) {
            state.syntax_error("Ambiguous exponentiation requires parentheses");
            return None;
        }
        Some(binary(state, left, operator, info.precedence, info.associativity, MapleElementType::BinaryExpr, |state, minimum| self.parse_pratt(state, minimum)))
    }
}
