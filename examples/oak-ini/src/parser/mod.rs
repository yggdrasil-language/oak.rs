/// Element types and categories for the INI language.
pub mod element_type;

use crate::{
    language::{IniLanguage, IniValueStyle},
    lexer::{IniLexer, token_type::IniTokenType},
};
use oak_core::{
    TextEdit,
    parser::{ParseCache, ParseOutput, Parser, parse_with_lexer},
    source::Source,
};

/// INI parser state.
pub(crate) type State<'a, S> = oak_core::parser::ParserState<'a, IniLanguage, S>;

/// INI parser implementation.
pub struct IniParser<'config> {
    /// The INI language configuration.
    pub(crate) config: &'config IniLanguage,
}

impl<'config> IniParser<'config> {
    /// Creates a new `IniParser` with the given configuration.
    pub fn new(config: &'config IniLanguage) -> Self {
        Self { config }
    }

    fn at_stop(&self, state: &State<'_, impl Source + ?Sized>) -> bool {
        !state.not_at_end() || state.at(IniTokenType::Eof)
    }

    /// Parses an INI table or array of tables.
    pub(crate) fn parse_table<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::errors::OakError> {
        let checkpoint = state.checkpoint();
        let kind = if state.at(IniTokenType::DoubleLeftBracket) {
            state.expect(IniTokenType::DoubleLeftBracket)?;
            self.parse_key(state)?;
            state.expect(IniTokenType::DoubleRightBracket)?;
            element_type::IniElementType::ArrayOfTables
        }
        else {
            state.expect(IniTokenType::LeftBracket)?;
            self.parse_key(state)?;
            state.expect(IniTokenType::RightBracket)?;
            element_type::IniElementType::Table
        };

        // Sections can have key-values following them
        while !self.at_stop(state) && !state.at(IniTokenType::LeftBracket) && !state.at(IniTokenType::DoubleLeftBracket) {
            self.skip_trivia(state);
            if self.at_stop(state) || state.at(IniTokenType::LeftBracket) || state.at(IniTokenType::DoubleLeftBracket) {
                break;
            }
            self.parse_key_value(state)?;
        }

        state.finish_at(checkpoint, kind);
        Ok(())
    }

    /// Parses an INI key-value pair.
    pub(crate) fn parse_key_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::errors::OakError> {
        let checkpoint = state.checkpoint();
        self.parse_key(state)?;

        self.skip_trivia(state);
        state.expect(IniTokenType::Equal)?;
        // Line-remainder values keep the end-of-line boundary; only skip spaces already emitted as trivia before the value token.
        if self.config.value_style != IniValueStyle::LineRemainder {
            self.skip_trivia(state);
        }
        else {
            while state.at(IniTokenType::Whitespace) {
                state.bump();
            }
        }

        self.parse_value(state)?;

        state.finish_at(checkpoint, element_type::IniElementType::KeyValue);
        Ok(())
    }

    /// Parses an INI key.
    fn parse_key<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::errors::OakError> {
        let checkpoint = state.checkpoint();
        // Support dotted keys: a.b.c
        loop {
            if state.at(IniTokenType::Identifier) || state.at(IniTokenType::String) {
                state.bump();
            }
            else if self.config.numeric_keys && (state.at(IniTokenType::Integer) || state.at(IniTokenType::Float)) {
                state.bump();
            }
            else {
                let err = oak_core::errors::OakError::expected_token("identifier or string", state.current_offset(), state.source_id());
                state.errors.push(err);
                return Err(state.errors.last().unwrap().clone());
            }

            self.skip_trivia(state);
            if state.at(IniTokenType::Dot) {
                state.bump();
                self.skip_trivia(state);
            }
            else {
                break;
            }
        }
        state.finish_at(checkpoint, element_type::IniElementType::Key);
        Ok(())
    }

    /// Parses an INI value.
    fn parse_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::errors::OakError> {
        let checkpoint = state.checkpoint();
        if self.config.value_style == IniValueStyle::LineRemainder {
            // Lexer emits a single String token (possibly empty) for the line remainder.
            if state.at(IniTokenType::String) {
                state.bump();
            }
            else if self.at_stop(state) || state.at(IniTokenType::Newline) {
                // `Key=` with nothing before EOF/newline — empty value node.
            }
            else {
                let err = oak_core::errors::OakError::expected_token("line value", state.current_offset(), state.source_id());
                state.errors.push(err);
                return Err(state.errors.last().unwrap().clone());
            }
            state.finish_at(checkpoint, element_type::IniElementType::Value);
            return Ok(());
        }

        let kind = state.peek_kind().ok_or_else(|| {
            let err = oak_core::errors::OakError::unexpected_eof(state.tokens.index(), state.source_id());
            state.errors.push(err);
            state.errors.last().unwrap().clone()
        })?;

        match kind {
            IniTokenType::Identifier | IniTokenType::String | IniTokenType::Integer | IniTokenType::Float | IniTokenType::Boolean | IniTokenType::DateTime => {
                state.bump();
            }
            IniTokenType::LeftBracket => {
                self.parse_array(state)?;
            }
            IniTokenType::LeftBrace => {
                self.parse_inline_table(state)?;
            }
            _ => {
                let err = oak_core::errors::OakError::expected_token("value", state.current_offset(), state.source_id());
                state.errors.push(err);
                return Err(state.errors.last().unwrap().clone());
            }
        }

        state.finish_at(checkpoint, element_type::IniElementType::Value);
        Ok(())
    }

    /// Parses an INI array.
    fn parse_array<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::errors::OakError> {
        let checkpoint = state.checkpoint();
        state.expect(IniTokenType::LeftBracket)?;
        self.skip_trivia(state);

        while !self.at_stop(state) && !state.at(IniTokenType::RightBracket) {
            self.parse_value(state)?;
            self.skip_trivia(state);
            if state.at(IniTokenType::Comma) {
                state.bump();
                self.skip_trivia(state);
            }
        }

        state.expect(IniTokenType::RightBracket)?;
        state.finish_at(checkpoint, element_type::IniElementType::Array);
        Ok(())
    }

    /// Parses an INI inline table.
    fn parse_inline_table<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::errors::OakError> {
        let checkpoint = state.checkpoint();
        state.expect(IniTokenType::LeftBrace)?;
        self.skip_trivia(state);

        while !self.at_stop(state) && !state.at(IniTokenType::RightBrace) {
            self.parse_key_value(state)?;
            self.skip_trivia(state);
            if state.at(IniTokenType::Comma) {
                state.bump();
                self.skip_trivia(state);
            }
        }

        state.expect(IniTokenType::RightBrace)?;
        state.finish_at(checkpoint, element_type::IniElementType::InlineTable);
        Ok(())
    }

    /// Skips trivia (whitespace, newlines, comments).
    fn skip_trivia<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while !self.at_stop(state) {
            let kind = match state.peek_kind() {
                Some(k) => k,
                None => break,
            };

            if kind == IniTokenType::Whitespace || kind == IniTokenType::Newline || kind == IniTokenType::Comment {
                state.bump();
            }
            else {
                break;
            }
        }
    }
}

impl<'config> Parser<IniLanguage> for IniParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<IniLanguage>) -> ParseOutput<'a, IniLanguage> {
        let lexer = IniLexer::new(self.config);
        parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            while !self.at_stop(state) {
                self.skip_trivia(state);
                if self.at_stop(state) {
                    break;
                }

                if state.at(IniTokenType::LeftBracket) || state.at(IniTokenType::DoubleLeftBracket) {
                    self.parse_table(state)?
                }
                else {
                    self.parse_key_value(state)?
                }
            }

            Ok(state.finish_at(checkpoint, element_type::IniElementType::Root))
        })
    }
}
