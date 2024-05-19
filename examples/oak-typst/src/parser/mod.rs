/// Element type definitions for Typst parser.
pub mod element_type;

use crate::{
    language::TypstLanguage,
    lexer::{TypstLexer, token_type::TypstTokenType},
    parser::element_type::TypstElementType,
};
use oak_core::{
    GreenNode, OakError,
    parser::{ParseCache, ParseOutput, Parser, ParserState, parse_with_lexer},
    source::{Source, TextEdit},
};

pub(crate) type State<'a, S> = ParserState<'a, TypstLanguage, S>;

/// Parser for Typst source code.
pub struct TypstParser<'config> {
    pub(crate) config: &'config TypstLanguage,
}

impl<'config> TypstParser<'config> {
    /// Creates a new TypstParser with the given language configuration.
    pub fn new(config: &'config TypstLanguage) -> Self {
        Self { config }
    }

    fn parse_item<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let kind = state.peek_kind();
        match kind {
            Some(TypstTokenType::Heading) => {
                let checkpoint = state.checkpoint();
                state.bump(); // Heading marker
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Newline) {
                    self.parse_item(state)?;
                }
                state.finish_at(checkpoint, TypstElementType::Heading);
            }
            Some(TypstTokenType::Hash) => {
                let checkpoint = state.checkpoint();
                state.bump(); // #
                // Check if it's "quote" or other commands
                while state.not_at_end()
                    && state
                        .peek_kind()
                        .map(|k| {
                            matches!(
                                k,
                                TypstTokenType::Identifier
                                    | TypstTokenType::Let
                                    | TypstTokenType::If
                                    | TypstTokenType::Else
                                    | TypstTokenType::For
                                    | TypstTokenType::While
                                    | TypstTokenType::Set
                                    | TypstTokenType::Show
                                    | TypstTokenType::Import
                                    | TypstTokenType::Include
                            )
                        })
                        .unwrap_or(false)
                {
                    state.bump()
                }

                if state.peek_kind() == Some(TypstTokenType::LeftBracket) {
                    state.bump(); // [
                    while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::RightBracket) {
                        self.parse_item(state)?;
                    }
                    if state.peek_kind() == Some(TypstTokenType::RightBracket) {
                        state.bump();
                    }
                }
                else {
                    // 没有参数列表的简单 `#cmd`，停在行尾；空白仍是 trivia，不会出现在 peek_kind 中
                    while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Newline) {
                        state.bump()
                    }
                }
                state.finish_at(checkpoint, TypstElementType::Quote);
            }
            Some(TypstTokenType::Dollar) => {
                let checkpoint = state.checkpoint();
                state.bump(); // $
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Dollar) {
                    self.parse_item(state)?;
                }
                if state.peek_kind() == Some(TypstTokenType::Dollar) {
                    state.bump()
                }
                state.finish_at(checkpoint, TypstElementType::Math);
            }
            Some(TypstTokenType::Strong) => {
                let checkpoint = state.checkpoint();
                state.bump(); // *
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Strong) {
                    self.parse_item(state)?;
                }
                if state.peek_kind() == Some(TypstTokenType::Strong) {
                    state.bump()
                }
                state.finish_at(checkpoint, TypstElementType::Strong);
            }
            Some(TypstTokenType::Emphasis) => {
                let checkpoint = state.checkpoint();
                state.bump(); // _
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Emphasis) {
                    self.parse_item(state)?;
                }
                if state.peek_kind() == Some(TypstTokenType::Emphasis) {
                    state.bump()
                }
                state.finish_at(checkpoint, TypstElementType::Emphasis);
            }
            Some(TypstTokenType::ListItem) => {
                let checkpoint = state.checkpoint();
                state.bump(); // - or +
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Newline) {
                    self.parse_item(state)?;
                }
                state.finish_at(checkpoint, TypstElementType::ListItem);
            }
            Some(TypstTokenType::EnumItem) => {
                let checkpoint = state.checkpoint();
                state.bump(); // 1.
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Newline) {
                    self.parse_item(state)?;
                }
                state.finish_at(checkpoint, TypstElementType::EnumItem);
            }
            Some(TypstTokenType::Backtick) => {
                let checkpoint = state.checkpoint();
                state.bump(); // `
                while state.not_at_end() && state.peek_kind() != Some(TypstTokenType::Backtick) {
                    state.bump()
                }
                if state.peek_kind() == Some(TypstTokenType::Backtick) {
                    state.bump()
                }
                state.finish_at(checkpoint, TypstElementType::Raw);
            }
            _ => {
                state.bump();
            }
        };
        Ok(())
    }
}

impl<'config> Parser<TypstLanguage> for TypstParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<TypstLanguage>) -> ParseOutput<'a, TypstLanguage> {
        let lexer = TypstLexer::new(&self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            while state.not_at_end() {
                self.parse_item(state)?
            }

            Ok(state.finish_at(checkpoint, TypstElementType::Root))
        })
    }
}
