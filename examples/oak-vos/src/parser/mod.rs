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
            if state.at(VosTokenType::Identifier) { state.bump(); }
            self.skip_trivia(state);
            if state.eat(VosTokenType::LeftBrace) {
                let mut depth = 1usize;
                while state.not_at_end() && depth > 0 {
                    match state.current().map(|token| token.kind) {
                        Some(VosTokenType::LeftBrace) => { depth += 1; state.bump(); }
                        Some(VosTokenType::RightBrace) => { depth -= 1; state.bump(); }
                        Some(_) => state.bump(),
                        None => break,
                    }
                }
                if depth != 0 { return Err(state.unexpected_eof()); }
            } else {
                while state.not_at_end() && !matches!(state.current().map(|token| token.kind), Some(VosTokenType::Semicolon)) { state.bump(); }
                state.eat(VosTokenType::Semicolon);
            }
            Ok(())
        })
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
                if matches!(state.current().map(|token| token.kind), Some(VosTokenType::Namespace | VosTokenType::Table | VosTokenType::Class | VosTokenType::Enums | VosTokenType::Flags | VosTokenType::Obsolete | VosTokenType::Using | VosTokenType::Const | VosTokenType::Service | VosTokenType::Udf | VosTokenType::Query)) {
                    self.parse_declaration(state)?;
                } else {
                    state.bump();
                }
            }
            Ok(state.finish_at(checkpoint, VosElementType::Root))
        })
    }
}
