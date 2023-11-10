use oak_core::{OakError, Parser, ParseCache, Source, TextEdit, TokenType, parser::parse_with_lexer};

use crate::{language::VosLanguage, lexer::{VosLexer, VosTokenType}};

/// CST element kinds.
pub mod element_type;
pub use element_type::VosElementType;

type State<'a, S> = oak_core::ParserState<'a, VosLanguage, S>;

/// Oak parser for the VOS surface syntax.
#[derive(Clone, Debug, Default)]
pub struct VosParser;

impl VosParser {
    /// Creates a parser using the VOS language configuration.
    pub fn new(_config: &VosLanguage) -> Self { Self }

    fn skip_trivia<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.current().map(|token| token.kind.is_ignored()).unwrap_or(false) { state.bump(); }
    }

    fn declaration_kind(kind: VosTokenType) -> VosElementType {
        match kind {
            VosTokenType::Namespace => VosElementType::Namespace,
            VosTokenType::Table => VosElementType::Table,
            VosTokenType::Class => VosElementType::Class,
            VosTokenType::Enums => VosElementType::Enums,
            VosTokenType::Flags => VosElementType::Flags,
            VosTokenType::Service => VosElementType::Service,
            VosTokenType::Micro => VosElementType::Micro,
            VosTokenType::Udf => VosElementType::Udf,
            VosTokenType::Query => VosElementType::Query,
            VosTokenType::Using => VosElementType::Using,
            VosTokenType::Const => VosElementType::Const,
            VosTokenType::Obsolete => VosElementType::Obsolete,
            _ => VosElementType::Error,
        }
    }

    fn parse_declaration<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let Some(token) = state.current() else { return Ok(()) };
        let element = Self::declaration_kind(token.kind);
        state.incremental_node(element, |state| {
            state.bump();
            self.skip_trivia(state);
            if element == VosElementType::Obsolete && state.at(VosTokenType::Table) {
                state.bump();
                self.skip_trivia(state);
            }
            if state.at(VosTokenType::Identifier) { state.bump(); }
            self.skip_trivia(state);
            while state.not_at_end() {
                let Some(kind) = state.current().map(|token| token.kind) else { break };
                if kind == VosTokenType::Semicolon { state.bump(); break; }
                if Self::declaration_kind(kind) != VosElementType::Error { break; }
                if kind == VosTokenType::LeftBrace {
                    self.parse_group(state)?;
                    break;
                }
                if matches!(kind, VosTokenType::LeftParen | VosTokenType::LeftBracket) {
                    self.parse_group(state)?;
                } else {
                    self.consume_token(state)?;
                }
            }
            Ok(())
        })
    }

    fn consume_token<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        if matches!(state.current().map(|token| token.kind), Some(VosTokenType::Error | VosTokenType::RightBrace | VosTokenType::RightParen | VosTokenType::RightBracket)) {
            state.record_unexpected_token("invalid VOS token or unmatched delimiter");
            return Err(state.errors.last().cloned().expect("Oak records the syntax error"));
        }
        state.bump();
        Ok(())
    }

    fn parse_group<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let checkpoint = state.checkpoint();
        let mut closings = Vec::new();
        let mut element = VosElementType::Block;
        loop {
            let Some(kind) = state.current().map(|token| token.kind) else { return Err(state.unexpected_eof()) };
            match kind {
                VosTokenType::LeftBrace => closings.push(VosTokenType::RightBrace),
                VosTokenType::LeftParen => {
                    if closings.is_empty() { element = VosElementType::Parentheses; }
                    closings.push(VosTokenType::RightParen);
                }
                VosTokenType::LeftBracket => {
                    if closings.is_empty() { element = VosElementType::Brackets; }
                    closings.push(VosTokenType::RightBracket);
                }
                VosTokenType::RightBrace | VosTokenType::RightParen | VosTokenType::RightBracket => {
                    if closings.pop() != Some(kind) {
                        state.record_unexpected_token("mismatched VOS delimiter");
                        return Err(state.errors.last().cloned().expect("Oak records the syntax error"));
                    }
                }
                VosTokenType::Eof => return Err(state.unexpected_eof()),
                VosTokenType::Error => {
                    state.record_unexpected_token("invalid VOS token");
                    return Err(state.errors.last().cloned().expect("Oak records the lexical error"));
                }
                _ => {}
            }
            state.bump();
            if closings.is_empty() { break; }
        }
        state.finish_at(checkpoint, element);
        Ok(())
    }
}

impl Parser<VosLanguage> for VosParser {
    fn parse<'a, S: Source + ?Sized>(&self, source: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<VosLanguage>) -> oak_core::ParseOutput<'a, VosLanguage> {
        let lexer = VosLexer;
        parse_with_lexer(&lexer, source, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            while state.not_at_end() {
                self.skip_trivia(state);
                if state.at(VosTokenType::Eof) { break; }
                if state.at(VosTokenType::Error) {
                    state.record_unexpected_token("invalid VOS token");
                    return Err(state.errors.last().cloned().expect("Oak records the lexical error"));
                }
                if state.current().map(|token| Self::declaration_kind(token.kind) != VosElementType::Error).unwrap_or(false) {
                    self.parse_declaration(state)?;
                } else if matches!(state.current().map(|token| token.kind), Some(VosTokenType::LeftBrace | VosTokenType::LeftParen | VosTokenType::LeftBracket)) {
                    self.parse_group(state)?;
                } else {
                    self.consume_token(state)?;
                }
            }
            Ok(state.finish_at(checkpoint, VosElementType::Root))
        })
    }
}
