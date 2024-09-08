#![doc = include_str!("readme.md")]

/// Element types for the DOT language syntax tree.
pub mod element_type;

use crate::{
    language::DotLanguage,
    lexer::{DotLexer, token_type::DotTokenType},
    parser::element_type::DotElementType,
};
use oak_core::{
    OakError,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

pub(crate) type State<'a, S> = ParserState<'a, DotLanguage, S>;

/// A parser for the DOT language.
pub struct DotParser<'config> {
    pub(crate) config: &'config DotLanguage,
}

impl<'config> DotParser<'config> {
    /// Creates a new `DotParser` with the given configuration.
    pub fn new(config: &'config DotLanguage) -> Self {
        Self { config }
    }

    /// Returns `true` if the current token can act as a DOT identifier.
    ///
    /// DOT keywords are not reserved in every position: an identifier slot accepts a
    /// keyword token as well, which keeps `A -> node` and `shape=box` parseable.
    fn at_id<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
        matches!(state.peek_kind(), Some(DotTokenType::Identifier | DotTokenType::String | DotTokenType::Number | DotTokenType::Graph | DotTokenType::Digraph | DotTokenType::Subgraph | DotTokenType::Node | DotTokenType::Edge | DotTokenType::Strict))
    }

    /// Returns `true` if the current token can start a plain (unquoted) identifier.
    fn at_plain_id<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
        matches!(state.peek_kind(), Some(DotTokenType::Identifier | DotTokenType::String | DotTokenType::Number))
    }

    /// Returns `true` if the current token can start an edge operand.
    fn at_operand_start<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
        Self::at_id(state) || state.at(DotTokenType::LeftBrace)
    }

    /// Returns `true` if the current token can start a statement.
    fn at_statement_start<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
        matches!(state.peek_kind(), Some(DotTokenType::Subgraph | DotTokenType::LeftBrace | DotTokenType::Identifier | DotTokenType::String | DotTokenType::Number | DotTokenType::Graph | DotTokenType::Digraph | DotTokenType::Node | DotTokenType::Edge))
    }

    /// Parses the whole file: a sequence of graph definitions plus any stray content.
    fn parse_root<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        state.skip_trivia();

        while state.not_at_end() && !state.at(DotTokenType::Eof) {
            if state.at(DotTokenType::Strict) || state.at(DotTokenType::Graph) || state.at(DotTokenType::Digraph) {
                self.parse_graph(state);
                continue;
            }

            let offset = state.current_offset();
            let text = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
            state.errors.push(OakError::unexpected_token(text, offset, state.source_id()));
            state.bump();
        }
    }

    /// Parses `[strict] (graph|digraph) [id] { statement_list }`.
    fn parse_graph<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();

        if state.at(DotTokenType::Strict) {
            state.bump();
        }

        if state.at(DotTokenType::Graph) || state.at(DotTokenType::Digraph) {
            state.bump();
        }
        else {
            state.errors.push(OakError::expected_token("`graph` or `digraph`", state.current_offset(), state.source_id()));
        }

        if Self::at_plain_id(state) {
            state.bump();
        }

        let has_body = if state.at(DotTokenType::LeftBrace) {
            state.bump();
            true
        }
        else {
            state.errors.push(OakError::expected_token("`{`", state.current_offset(), state.source_id()));
            false
        };

        if has_body {
            self.parse_statement_list(state);
            if state.at(DotTokenType::RightBrace) {
                state.bump();
            }
            else {
                state.record_unexpected_eof();
            }
        }

        state.finish_at(checkpoint, DotElementType::GraphDefinition);
    }

    /// Parses statements until the closing brace, the end of the file, or a hard stop.
    fn parse_statement_list<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        loop {
            if !state.not_at_end() || state.at(DotTokenType::Eof) || state.at(DotTokenType::RightBrace) {
                break;
            }

            if state.at(DotTokenType::Semicolon) {
                state.bump();
                continue;
            }

            if !Self::at_statement_start(state) {
                let offset = state.current_offset();
                let text = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
                state.errors.push(OakError::unexpected_token(text, offset, state.source_id()));
                state.bump();
                self.recover_statement(state);
                continue;
            }

            self.parse_statement(state);

            while state.at(DotTokenType::Semicolon) {
                state.bump();
            }
        }
    }

    /// Skips broken input until the next terminator, block boundary, or statement start.
    fn recover_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        loop {
            if !state.not_at_end() || state.at(DotTokenType::Eof) {
                break;
            }
            if state.at(DotTokenType::Semicolon) {
                state.bump();
                break;
            }
            if state.at(DotTokenType::RightBrace) || Self::at_statement_start(state) {
                break;
            }
            state.bump();
        }
    }

    /// Parses a single statement, always wrapping its tokens into one composite element.
    fn parse_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        match state.peek_kind() {
            Some(DotTokenType::Subgraph) | Some(DotTokenType::LeftBrace) => {
                let checkpoint = state.checkpoint();
                self.parse_subgraph(state);

                if state.at(DotTokenType::Arrow) || state.at(DotTokenType::Line) {
                    self.parse_edge_chain(state);
                    self.parse_trailing_attributes(state);
                    state.finish_at(checkpoint, DotElementType::GraphEdge);
                }
            }
            Some(DotTokenType::Graph) | Some(DotTokenType::Digraph) | Some(DotTokenType::Node) | Some(DotTokenType::Edge) if state.peek_non_trivia_kind_at(1) == Some(DotTokenType::LeftBracket) => {
                self.parse_attribute_statement(state);
            }
            Some(_) => self.parse_node_or_edge(state),
            None => {}
        }
    }

    /// Parses `(node_id|subgraph) (edgeop (node_id|subgraph))* [attr_list]` or a node statement.
    fn parse_node_or_edge<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();

        // `id = value` is an assignment, not a node statement.
        if state.peek_non_trivia_kind_at(1) == Some(DotTokenType::Equal) {
            state.bump();

            if state.eat(DotTokenType::Equal) && Self::at_id(state) {
                state.bump();
            }
            else {
                state.errors.push(OakError::expected_token("an assignment value", state.current_offset(), state.source_id()));
            }

            state.finish_at(checkpoint, DotElementType::GraphAssignment);
            return;
        }

        self.parse_node_id(state);
        let has_edge = self.parse_edge_chain(state);
        self.parse_trailing_attributes(state);

        let kind = if has_edge { DotElementType::GraphEdge } else { DotElementType::GraphNode };
        state.finish_at(checkpoint, kind);
    }

    /// Parses `edgeop (node_id|subgraph)` pairs and folds them into the current statement.
    ///
    /// Returns `true` when at least one complete hop was consumed. A dangling operator is
    /// left outside the statement so recovery can surface it as a diagnostic.
    fn parse_edge_chain<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let mut has_edge = false;

        loop {
            if !state.at(DotTokenType::Arrow) && !state.at(DotTokenType::Line) {
                break;
            }

            let operator_checkpoint = state.checkpoint();
            let operator_offset = state.current_offset();
            state.bump();

            if !Self::at_operand_start(state) {
                state.restore(operator_checkpoint);
                state.errors.push(OakError::syntax_error("expected a node or a subgraph after the edge operator", operator_offset, state.source_id()));
                break;
            }

            if state.at(DotTokenType::Subgraph) || state.at(DotTokenType::LeftBrace) {
                self.parse_subgraph(state);
            }
            else {
                self.parse_node_id(state);
            }
            has_edge = true;
        }

        has_edge
    }

    /// Parses one or more attribute lists following a node or an edge.
    fn parse_trailing_attributes<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.at(DotTokenType::LeftBracket) {
            self.parse_attribute_list(state);
        }
    }

    /// Parses `node_id [ ':' id [ ':' compass ] ]`.
    fn parse_node_id<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        if !Self::at_id(state) {
            return false;
        }

        let checkpoint = state.checkpoint();
        state.bump();

        if state.at(DotTokenType::Colon) {
            self.parse_port(state);
        }

        state.finish_at(checkpoint, DotElementType::NodeId);
        true
    }

    /// Parses a port suffix, accepting both `:id` and `:id:compass` forms.
    fn parse_port<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // `:`

        if !Self::at_id(state) {
            state.errors.push(OakError::expected_token("a port identifier or a compass point", state.current_offset(), state.source_id()));
            state.finish_at(checkpoint, DotElementType::Port);
            return;
        }
        state.bump();

        if state.at(DotTokenType::Colon) {
            state.bump();
            if Self::at_id(state) {
                state.bump();
            }
            else {
                state.errors.push(OakError::expected_token("a compass point", state.current_offset(), state.source_id()));
            }

            while state.at(DotTokenType::Colon) {
                state.errors.push(OakError::syntax_error("a port accepts at most two `:` separated segments", state.current_offset(), state.source_id()));
                state.bump();
                if Self::at_id(state) {
                    state.bump();
                }
            }
        }

        state.finish_at(checkpoint, DotElementType::Port);
    }

    /// Parses `graph|node|edge [attr_list]`.
    fn parse_attribute_statement<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump();

        if state.at(DotTokenType::LeftBracket) {
            self.parse_attribute_list(state);
        }
        else {
            state.errors.push(OakError::expected_token("`[`", state.current_offset(), state.source_id()));
        }

        state.finish_at(checkpoint, DotElementType::GraphAttribute);
    }

    /// Parses `[ a_list ]`, tolerating missing separators and an unterminated list.
    fn parse_attribute_list<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();

        if !state.eat(DotTokenType::LeftBracket) {
            state.errors.push(OakError::expected_token("`[`", state.current_offset(), state.source_id()));
        }

        loop {
            if !state.not_at_end() || state.at(DotTokenType::Eof) || state.at(DotTokenType::RightBrace) || state.at(DotTokenType::RightBracket) {
                break;
            }

            if state.at(DotTokenType::Error) {
                self.record_lexical_error(state);
                continue;
            }

            if !Self::at_id(state) {
                let offset = state.current_offset();
                let text = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
                state.errors.push(OakError::unexpected_token(text, offset, state.source_id()));
                state.bump();
                continue;
            }

            state.bump();

            if state.eat(DotTokenType::Equal) {
                if Self::at_id(state) {
                    state.bump();
                }
                else if state.at(DotTokenType::Error) {
                    self.record_lexical_error(state);
                }
                else {
                    state.errors.push(OakError::expected_token("an attribute value", state.current_offset(), state.source_id()));
                }
            }

            if state.at(DotTokenType::Comma) || state.at(DotTokenType::Semicolon) {
                state.bump();
            }
        }

        if !state.eat(DotTokenType::RightBracket) {
            state.errors.push(OakError::expected_token("`]`", state.current_offset(), state.source_id()));
        }

        state.finish_at(checkpoint, DotElementType::AttributeList);
    }

    /// Records a diagnostic for an error token emitted by the lexer.
    fn record_lexical_error<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let offset = state.current_offset();
        let text = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
        let message = if text.starts_with('"') { "unterminated string literal" } else { "unexpected character" };
        state.errors.push(OakError::syntax_error(message, offset, state.source_id()));
        state.bump();
    }

    /// Parses `[subgraph] [id] { statement_list }`, including anonymous subgraph operands.
    fn parse_subgraph<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();

        if state.at(DotTokenType::Subgraph) {
            state.bump();
            if Self::at_plain_id(state) {
                state.bump();
            }
        }

        if !state.at(DotTokenType::LeftBrace) {
            state.errors.push(OakError::expected_token("`{`", state.current_offset(), state.source_id()));
            state.finish_at(checkpoint, DotElementType::GraphSubgraph);
            return;
        }
        state.bump();

        self.parse_statement_list(state);

        if state.at(DotTokenType::RightBrace) {
            state.bump();
        }
        else {
            state.record_unexpected_eof();
        }

        state.finish_at(checkpoint, DotElementType::GraphSubgraph);
    }
}

impl<'config> Parser<DotLanguage> for DotParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<DotLanguage>) -> ParseOutput<'a, DotLanguage> {
        let lexer = DotLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            self.parse_root(state);
            Ok(state.finish_at(checkpoint, DotElementType::Root))
        })
    }
}
