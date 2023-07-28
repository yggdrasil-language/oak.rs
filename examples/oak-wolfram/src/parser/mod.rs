#![doc = include_str!("readme.md")]

/// Wolfram element types.
pub mod element_type;

use crate::{
    language::WolframLanguage,
    lexer::{WolframLexer, token_type::WolframTokenType},
    parser::element_type::WolframElementType,
};
use oak_core::{
    parser::{OperatorInfo, ParseCache, ParseOutput, Parser, ParserState, Pratt, PrattParser, binary, parse_with_lexer, postfix, unary},
    source::{Source, TextEdit},
    tree::GreenNode,
};

pub(crate) type State<'a, S> = ParserState<'a, WolframLanguage, S>;

/// Parser for the Wolfram language.
#[derive(Debug, Clone)]
pub struct WolframParser<'config> {
    /// The Wolfram language configuration.
    config: &'config WolframLanguage,
}

impl<'config> WolframParser<'config> {
    /// Creates a new `WolframParser` with the given configuration.
    pub fn new(config: &'config WolframLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<WolframLanguage> for WolframParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<WolframLanguage>) -> ParseOutput<'a, WolframLanguage> {
        let lexer = WolframLexer::new(&self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            while state.not_at_end() && state.not_at(WolframTokenType::Eof) {
                self.parse_expression(state);
                if state.at(WolframTokenType::Semicolon) {
                    state.bump();
                }
            }

            Ok(state.finish_at(checkpoint, WolframElementType::Root))
        })
    }
}

impl<'config> WolframParser<'config> {
    fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        self.parse_pratt(state, 0);
    }

    fn parse_pratt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, min_precedence: u8) -> &'a GreenNode<'a, WolframLanguage> {
        PrattParser::new(self.clone()).parse_expr(state, min_precedence)
    }

    fn parse_arguments<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // [

        while state.not_at(WolframTokenType::RightBracket) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(WolframTokenType::Comma) {
                state.bump();
            }
        }

        if state.at(WolframTokenType::RightBracket) {
            state.bump();
        }
        state.finish_at(checkpoint, WolframElementType::Arguments);
    }

    fn parse_list<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WolframLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // {

        while state.not_at(WolframTokenType::RightBrace) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(WolframTokenType::Comma) {
                state.bump();
            }
        }

        if state.at(WolframTokenType::RightBrace) {
            state.bump();
        }
        state.finish_at(checkpoint, WolframElementType::List)
    }

    /// Keyword heads that are still ordinary Wolfram symbols (`If[…]`, `Import[…]`, `True`, …).
    fn is_symbol_token(kind: WolframTokenType) -> bool {
        matches!(
            kind,
            WolframTokenType::Identifier
                | WolframTokenType::If
                | WolframTokenType::Then
                | WolframTokenType::Else
                | WolframTokenType::While
                | WolframTokenType::For
                | WolframTokenType::Do
                | WolframTokenType::Function
                | WolframTokenType::Module
                | WolframTokenType::Block
                | WolframTokenType::With
                | WolframTokenType::Table
                | WolframTokenType::Map
                | WolframTokenType::Apply
                | WolframTokenType::Select
                | WolframTokenType::Cases
                | WolframTokenType::Rule
                | WolframTokenType::RuleDelayed
                | WolframTokenType::Set
                | WolframTokenType::SetDelayed
                | WolframTokenType::Unset
                | WolframTokenType::Clear
                | WolframTokenType::ClearAll
                | WolframTokenType::Return
                | WolframTokenType::Break
                | WolframTokenType::Continue
                | WolframTokenType::True
                | WolframTokenType::False
                | WolframTokenType::Null
                | WolframTokenType::Export
                | WolframTokenType::Import
        )
    }

    /// `expr[[…]]` Part, or `expr[…]` Call on a non-primary head (e.g. list apply).
    fn parse_bracket_postfix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, WolframLanguage>) -> &'a GreenNode<'a, WolframLanguage> {
        let checkpoint = state.checkpoint_before(left);
        // Distinguish Part `[[` from Call `[`.
        state.bump(); // first [
        let is_part = state.at(WolframTokenType::LeftBracket);
        if is_part {
            state.bump(); // second [
        }

        // Reuse argument-list body without consuming an extra opening bracket.
        let args_checkpoint = state.checkpoint();
        while state.not_at(WolframTokenType::RightBracket) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(WolframTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(WolframTokenType::RightBracket) {
            state.bump();
        }
        if is_part && state.at(WolframTokenType::RightBracket) {
            state.bump();
        }
        state.finish_at(args_checkpoint, WolframElementType::Arguments);

        if is_part { state.finish_at(checkpoint, WolframElementType::Part) } else { state.finish_at(checkpoint, WolframElementType::Call) }
    }
}

impl<'config> Pratt<WolframLanguage> for WolframParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WolframLanguage> {
        let checkpoint = state.checkpoint();
        let kind = state.peek_kind();

        if kind.is_some_and(Self::is_symbol_token) {
            state.bump();
            if state.at(WolframTokenType::LeftBracket) && state.peek_non_trivia_kind_at(1) == Some(WolframTokenType::LeftBracket) {
                // `sym[[…]]` — finish symbol, then Part postfix (may repeat).
                let mut node = state.finish_at(checkpoint, WolframElementType::Symbol);
                while state.at(WolframTokenType::LeftBracket) {
                    node = self.parse_bracket_postfix(state, node);
                }
                node
            }
            else if state.at(WolframTokenType::LeftBracket) {
                // `sym[…]` / `sym[…][…]` Call groups (single brackets only).
                while state.at(WolframTokenType::LeftBracket) && state.peek_non_trivia_kind_at(1) != Some(WolframTokenType::LeftBracket) {
                    self.parse_arguments(state);
                }
                let mut node = state.finish_at(checkpoint, WolframElementType::Call);
                while state.at(WolframTokenType::LeftBracket) {
                    node = self.parse_bracket_postfix(state, node);
                }
                node
            }
            else {
                state.finish_at(checkpoint, WolframElementType::Symbol)
            }
        }
        else if state.at(WolframTokenType::Integer) || state.at(WolframTokenType::Real) || state.at(WolframTokenType::String) {
            state.bump();
            state.finish_at(checkpoint, WolframElementType::Literal)
        }
        else if state.at(WolframTokenType::LeftBrace) {
            self.parse_list(state)
        }
        else if state.at(WolframTokenType::Slot) || state.at(WolframTokenType::SlotSequence) {
            state.bump();
            state.finish_at(checkpoint, WolframElementType::Symbol)
        }
        else if state.at(WolframTokenType::Underscore) || state.at(WolframTokenType::DoubleUnderscore) || state.at(WolframTokenType::TripleUnderscore) {
            // `_` / `__` / `___`, optionally typed `_Integer`.
            state.bump();
            if state.peek_kind().is_some_and(Self::is_symbol_token) && state.peek_non_trivia_kind_at(1) != Some(WolframTokenType::LeftBracket) {
                state.bump();
            }
            state.finish_at(checkpoint, WolframElementType::Blank)
        }
        else if state.at(WolframTokenType::LeftParen) {
            state.bump();
            self.parse_expression(state);
            if state.at(WolframTokenType::RightParen) {
                state.bump();
            }
            state.finish_at(checkpoint, WolframElementType::Expression)
        }
        else {
            // Error handling
            state.bump();
            state.finish_at(checkpoint, WolframElementType::Error)
        }
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, WolframLanguage> {
        let kind = match state.peek_kind() {
            Some(k) => k,
            None => return self.primary(state),
        };

        let info = match kind {
            // Unary Minus sits between Times (90) and Power (120), matching Wolfram:
            // `-x^2` → Times[-1, Power[x, 2]], not Power[Times[-1, x], 2].
            WolframTokenType::Minus => Some(OperatorInfo::right(100)),
            // Logical Not (`!x`) stays below relational ops.
            WolframTokenType::Factorial => Some(OperatorInfo::right(65)),
            _ => None,
        };

        if let Some(info) = info { unary(state, kind, info.precedence, WolframElementType::PrefixExpr, |s, p| self.parse_pratt(s, p)) } else { self.primary(state) }
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, WolframLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, WolframLanguage>> {
        let kind = state.peek_kind()?;

        // Part `expr[[…]]` / Call `expr[…]` (high precedence postfix-like)
        if kind == WolframTokenType::LeftBracket {
            const BRACKET_PREC: u8 = 170;
            if BRACKET_PREC < min_precedence {
                return None;
            }
            return Some(self.parse_bracket_postfix(state, left));
        }

        // Postfix operators
        let postfix_info = match kind {
            WolframTokenType::Ampersand => Some(OperatorInfo::left(10)),  // body &
            WolframTokenType::Factorial => Some(OperatorInfo::left(160)), // x!
            // `x_` Pattern[x, Blank[]] — high precedence, below Part/Call.
            WolframTokenType::Underscore | WolframTokenType::DoubleUnderscore | WolframTokenType::TripleUnderscore => Some(OperatorInfo::left(165)),
            _ => None,
        };

        if let Some(info) = postfix_info {
            if info.precedence < min_precedence {
                return None;
            }
            let element = match kind {
                WolframTokenType::Underscore | WolframTokenType::DoubleUnderscore | WolframTokenType::TripleUnderscore => WolframElementType::Pattern,
                _ => WolframElementType::PostfixExpr,
            };
            return Some(postfix(state, left, kind, element));
        }

        // Binary/Infix operators
        let info = match kind {
            WolframTokenType::Semicolon => Some(OperatorInfo::left(5)), // a;b CompoundExpression
            WolframTokenType::Assign | WolframTokenType::Set | WolframTokenType::SetDelayed => Some(OperatorInfo::right(20)),
            WolframTokenType::Rule | WolframTokenType::RuleDelayed | WolframTokenType::Arrow | WolframTokenType::RuleDelayedOp => Some(OperatorInfo::right(30)),
            WolframTokenType::SlashSlash => Some(OperatorInfo::left(40)), // x // f
            WolframTokenType::Or => Some(OperatorInfo::left(50)),
            WolframTokenType::And => Some(OperatorInfo::left(60)),
            WolframTokenType::Equal | WolframTokenType::NotEqual | WolframTokenType::Less | WolframTokenType::Greater | WolframTokenType::LessEqual | WolframTokenType::GreaterEqual => Some(OperatorInfo::none(70)),
            WolframTokenType::Plus | WolframTokenType::Minus => Some(OperatorInfo::left(80)),
            WolframTokenType::Times | WolframTokenType::Divide => Some(OperatorInfo::left(90)),
            WolframTokenType::At => Some(OperatorInfo::right(100)),                 // f @ x
            WolframTokenType::MapOperator => Some(OperatorInfo::right(110)),        // f /@ list
            WolframTokenType::ApplyOperator => Some(OperatorInfo::right(110)),      // f @@ terms
            WolframTokenType::ApplyLevelOperator => Some(OperatorInfo::right(110)), // f @@@ terms
            WolframTokenType::MapAllOperator => Some(OperatorInfo::right(110)),     // f //@ list
            WolframTokenType::Power => Some(OperatorInfo::right(120)),
            _ => None,
        };

        if let Some(info) = info {
            if info.precedence < min_precedence {
                return None;
            }
            return Some(binary(state, left, kind, info.precedence, info.associativity, WolframElementType::BinaryExpr, |s, p| {
                self.parse_pratt(s, p)
            }));
        }

        // Implicit Times: `x y`, `2 x`, `Sin[x] Cos[x]` (Wolfram juxtaposition).
        const TIMES_PREC: u8 = 90;
        if TIMES_PREC >= min_precedence && Self::can_start_juxtaposition(state.peek_kind()) {
            let cp = state.checkpoint_before(left);
            let _right = self.parse_pratt(state, TIMES_PREC + 1);
            return Some(state.finish_at(cp, WolframElementType::BinaryExpr));
        }

        None
    }
}

impl<'config> WolframParser<'config> {
    /// Tokens that may begin a primary after juxtaposition (no leading infix op).
    fn can_start_juxtaposition(kind: Option<WolframTokenType>) -> bool {
        let Some(kind) = kind else {
            return false;
        };
        if Self::is_symbol_token(kind) {
            return true;
        }
        matches!(
            kind,
            WolframTokenType::Integer
                | WolframTokenType::Real
                | WolframTokenType::String
                | WolframTokenType::LeftBrace
                | WolframTokenType::LeftParen
                | WolframTokenType::Slot
                | WolframTokenType::SlotSequence
                | WolframTokenType::Underscore
                | WolframTokenType::DoubleUnderscore
                | WolframTokenType::TripleUnderscore
                | WolframTokenType::Minus // `-y` after `x` → Times[x, Times[-1, y]]
                | WolframTokenType::Factorial // `!p` after `x`
        )
    }
}
