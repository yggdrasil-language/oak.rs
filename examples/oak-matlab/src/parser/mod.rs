//! Matlab Pratt expression parser (arithmetic / calls / arrays / control).

/// Element kinds.
pub mod element_type;

use crate::{
    language::MatlabLanguage,
    lexer::{MatlabLexer, token_type::MatlabTokenType},
    parser::element_type::MatlabElementType,
};
use oak_core::{
    parser::{OperatorInfo, ParseCache, ParseOutput, Parser, ParserState, Pratt, PrattParser, binary, parse_with_lexer, postfix, unary},
    source::{Source, TextEdit},
    tree::GreenNode,
};

type State<'a, S> = ParserState<'a, MatlabLanguage, S>;

/// MATLAB parser.
#[derive(Debug, Clone)]
pub struct MatlabParser<'config> {
    /// Language configuration.
    config: &'config MatlabLanguage,
}

impl<'config> MatlabParser<'config> {
    /// Creates a new parser.
    pub fn new(config: &'config MatlabLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<MatlabLanguage> for MatlabParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<MatlabLanguage>) -> ParseOutput<'a, MatlabLanguage> {
        let lexer = MatlabLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            while state.not_at_end() && state.not_at(MatlabTokenType::Eof) {
                self.parse_statement(state);
                self.skip_statement_separators(state);
            }
            Ok(state.finish_at(checkpoint, MatlabElementType::Root))
        })
    }
}

impl<'config> MatlabParser<'config> {
    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        match state.peek_kind() {
            Some(MatlabTokenType::If) => self.parse_if(state),
            Some(MatlabTokenType::While) => self.parse_while(state),
            Some(MatlabTokenType::For) => self.parse_for(state),
            Some(MatlabTokenType::Switch) => self.parse_switch(state),
            Some(MatlabTokenType::Try) => self.parse_try(state),
            _ => self.parse_expression(state),
        }
    }

    fn parse_expression<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        self.parse_pratt(state, 0)
    }

    fn parse_pratt<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, min_precedence: u8) -> &'a GreenNode<'a, MatlabLanguage> {
        PrattParser::new(self.clone()).parse_expr(state, min_precedence)
    }

    fn skip_statement_separators<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.at(MatlabTokenType::Semicolon) || state.at(MatlabTokenType::Comma) {
            state.bump();
        }
    }

    fn at_block_terminator(state: &State<'_, impl Source + ?Sized>) -> bool {
        state.at(MatlabTokenType::End) || state.at(MatlabTokenType::Else) || state.at(MatlabTokenType::Elseif) || state.at(MatlabTokenType::Catch) || state.at(MatlabTokenType::Eof) || !state.not_at_end()
    }

    fn parse_block_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        self.skip_statement_separators(state);
        while !Self::at_block_terminator(state) {
            self.parse_statement(state);
            self.skip_statement_separators(state);
        }
    }

    fn parse_if<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // if
        self.parse_expression(state); // condition
        self.parse_block_body(state); // then
        if state.at(MatlabTokenType::Elseif) {
            // Keep elseif chain as nested IfStmt children for now.
            while state.at(MatlabTokenType::Elseif) {
                state.bump();
                self.parse_expression(state);
                self.parse_block_body(state);
            }
        }
        if state.at(MatlabTokenType::Else) {
            state.bump();
            self.parse_block_body(state);
        }
        if state.at(MatlabTokenType::End) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::IfStmt)
    }

    fn parse_while<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // while
        self.parse_expression(state);
        self.parse_block_body(state);
        if state.at(MatlabTokenType::End) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::WhileStmt)
    }

    fn parse_for<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // for
        self.parse_expression(state); // usually `i = 1:n`
        self.parse_block_body(state);
        if state.at(MatlabTokenType::End) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::ForStmt)
    }

    fn parse_switch<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // switch
        self.parse_expression(state); // discriminant
        self.skip_statement_separators(state);
        while state.at(MatlabTokenType::Case) {
            state.bump();
            self.parse_expression(state); // case value
            self.parse_switch_arm_body(state);
        }
        if state.at(MatlabTokenType::Otherwise) {
            state.bump();
            self.parse_switch_arm_body(state);
        }
        if state.at(MatlabTokenType::End) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::SwitchStmt)
    }

    fn parse_switch_arm_body<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        self.skip_statement_separators(state);
        while !Self::at_switch_arm_terminator(state) {
            self.parse_statement(state);
            self.skip_statement_separators(state);
        }
    }

    fn at_switch_arm_terminator(state: &State<'_, impl Source + ?Sized>) -> bool {
        state.at(MatlabTokenType::Case)
            || state.at(MatlabTokenType::Otherwise)
            || state.at(MatlabTokenType::End)
            || state.at(MatlabTokenType::Eof)
            || !state.not_at_end()
    }

    fn parse_try<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // try
        self.parse_block_body(state);
        if state.at(MatlabTokenType::Catch) {
            state.bump();
            // Optional catch identifier (`catch ME`) before catch body.
            if state.at(MatlabTokenType::Identifier) {
                self.parse_expression(state);
            }
            self.parse_block_body(state);
        }
        if state.at(MatlabTokenType::End) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::TryStmt)
    }

    fn parse_call_args<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // (
        while state.not_at(MatlabTokenType::RightParen) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(MatlabTokenType::Comma) {
                state.bump();
            }
        }
        if state.at(MatlabTokenType::RightParen) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::Arguments);
    }

    /// `expr(…)` call / indexing postfix.
    fn parse_paren_postfix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, MatlabLanguage>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint_before(left);
        self.parse_call_args(state);
        state.finish_at(checkpoint, MatlabElementType::Call)
    }

    fn parse_array<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // [
        while state.not_at(MatlabTokenType::RightBracket) && state.not_at_end() {
            self.parse_expression(state);
            if state.at(MatlabTokenType::Comma) || state.at(MatlabTokenType::Semicolon) {
                state.bump();
            }
        }
        if state.at(MatlabTokenType::RightBracket) {
            state.bump();
        }
        state.finish_at(checkpoint, MatlabElementType::Array)
    }

    /// `@sin` or `@(x,y) body`.
    fn parse_at_expr<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();
        state.bump(); // @
        if state.at(MatlabTokenType::LeftParen) {
            self.parse_call_args(state); // parameters
            self.parse_pratt(state, 0); // body
            state.finish_at(checkpoint, MatlabElementType::AnonymousFunction)
        }
        else {
            // `@name` — primary name / call target without consuming trailing `(args)` as the handle itself.
            if state.at(MatlabTokenType::Identifier) {
                state.bump();
            }
            else {
                self.primary(state);
            }
            state.finish_at(checkpoint, MatlabElementType::FunctionHandle)
        }
    }
}

impl<'config> Pratt<MatlabLanguage> for MatlabParser<'config> {
    fn primary<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let checkpoint = state.checkpoint();

        if state.at(MatlabTokenType::Identifier) {
            state.bump();
            if state.at(MatlabTokenType::LeftParen) {
                while state.at(MatlabTokenType::LeftParen) {
                    self.parse_call_args(state);
                }
                let mut node = state.finish_at(checkpoint, MatlabElementType::Call);
                while state.at(MatlabTokenType::LeftParen) {
                    node = self.parse_paren_postfix(state, node);
                }
                node
            }
            else {
                state.finish_at(checkpoint, MatlabElementType::Symbol)
            }
        }
        else if state.at(MatlabTokenType::End) {
            // Indexing `end` (and bare keyword use) as a symbol primary.
            state.bump();
            state.finish_at(checkpoint, MatlabElementType::Symbol)
        }
        else if state.at(MatlabTokenType::Colon) {
            // Lone `:` in subsref means "all" (e.g. `A(1,:)`).
            state.bump();
            state.finish_at(checkpoint, MatlabElementType::Symbol)
        }
        else if state.at(MatlabTokenType::At) {
            self.parse_at_expr(state)
        }
        else if state.at(MatlabTokenType::Number) || state.at(MatlabTokenType::String) || state.at(MatlabTokenType::Character) {
            state.bump();
            state.finish_at(checkpoint, MatlabElementType::Literal)
        }
        else if state.at(MatlabTokenType::LeftBracket) {
            let mut node = self.parse_array(state);
            while state.at(MatlabTokenType::LeftParen) {
                node = self.parse_paren_postfix(state, node);
            }
            node
        }
        else if state.at(MatlabTokenType::LeftParen) {
            state.bump();
            self.parse_expression(state);
            if state.at(MatlabTokenType::RightParen) {
                state.bump();
            }
            state.finish_at(checkpoint, MatlabElementType::Expression)
        }
        else {
            state.bump();
            state.finish_at(checkpoint, MatlabElementType::Error)
        }
    }

    fn prefix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> &'a GreenNode<'a, MatlabLanguage> {
        let kind = match state.peek_kind() {
            Some(k) => k,
            None => return self.primary(state),
        };
        let info = match kind {
            MatlabTokenType::Minus | MatlabTokenType::Plus | MatlabTokenType::Not => Some(OperatorInfo::right(150)),
            _ => None,
        };
        if let Some(info) = info { unary(state, kind, info.precedence, MatlabElementType::PrefixExpr, |s, p| self.parse_pratt(s, p)) } else { self.primary(state) }
    }

    fn infix<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, left: &'a GreenNode<'a, MatlabLanguage>, min_precedence: u8) -> Option<&'a GreenNode<'a, MatlabLanguage>> {
        let kind = state.peek_kind()?;

        // Indexing / call `expr(…)` (high precedence)
        if kind == MatlabTokenType::LeftParen {
            const PAREN_PREC: u8 = 170;
            if PAREN_PREC < min_precedence {
                return None;
            }
            return Some(self.parse_paren_postfix(state, left));
        }

        let postfix_info = match kind {
            MatlabTokenType::Transpose | MatlabTokenType::DotTranspose => Some(OperatorInfo::left(160)),
            _ => None,
        };
        if let Some(info) = postfix_info {
            if info.precedence < min_precedence {
                return None;
            }
            return Some(postfix(state, left, kind, MatlabElementType::PostfixExpr));
        }

        let info = match kind {
            MatlabTokenType::Assign => Some(OperatorInfo::right(20)),
            // MATLAB: `|`/`||` below `&`/`&&`; both elementwise and short-circuit forms are infix.
            MatlabTokenType::OrOr => Some(OperatorInfo::left(40)),
            MatlabTokenType::Or => Some(OperatorInfo::left(45)),
            MatlabTokenType::AndAnd => Some(OperatorInfo::left(50)),
            MatlabTokenType::And => Some(OperatorInfo::left(55)),
            MatlabTokenType::Equal | MatlabTokenType::NotEqual | MatlabTokenType::Less | MatlabTokenType::Greater | MatlabTokenType::LessEqual | MatlabTokenType::GreaterEqual => Some(OperatorInfo::none(60)),
            // MATLAB `a:b:c` is right-associative enough that left-assoc nesting is fixed in lowering.
            MatlabTokenType::Colon => Some(OperatorInfo::left(70)),
            MatlabTokenType::Plus | MatlabTokenType::Minus => Some(OperatorInfo::left(80)),
            MatlabTokenType::Times | MatlabTokenType::Divide | MatlabTokenType::LeftDivide | MatlabTokenType::DotTimes | MatlabTokenType::DotDivide | MatlabTokenType::DotLeftDivide => Some(OperatorInfo::left(90)),
            MatlabTokenType::Power | MatlabTokenType::DotPower => Some(OperatorInfo::right(120)),
            _ => None,
        }?;

        if info.precedence < min_precedence {
            return None;
        }

        Some(binary(state, left, kind, info.precedence, info.associativity, MatlabElementType::BinaryExpr, |s, p| self.parse_pratt(s, p)))
    }
}
