#![doc = include_str!("readme.md")]

/// Token kinds for athena.
pub mod token_type;

use crate::{language::AthenaLanguage, lexer::token_type::AthenaTokenType};
use oak_core::{
    Lexer, LexerCache, LexerState, OakError,
    lexer::{LexOutput, WhitespaceConfig},
    source::{Source, TextEdit},
};
use std::sync::LazyLock;

type State<'a, S> = LexerState<'a, S, AthenaLanguage>;

static ATHENA_WHITESPACE: LazyLock<WhitespaceConfig> = LazyLock::new(|| WhitespaceConfig {
    unicode_whitespace: true,
});

/// Lexer for the athena DSL.
#[derive(Clone, Debug)]
pub struct AthenaLexer<'config> {
    /// Language configuration.
    config: &'config AthenaLanguage,
}

impl<'config> Lexer<AthenaLanguage> for AthenaLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(
        &self,
        source: &S,
        _edits: &[TextEdit],
        cache: &'a mut impl LexerCache<AthenaLanguage>,
    ) -> LexOutput<AthenaLanguage> {
        let mut state = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> AthenaLexer<'config> {
    /// Creates a new lexer.
    pub fn new(config: &'config AthenaLanguage) -> Self {
        Self { config }
    }

    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        let _ = self.config;
        while state.not_at_end() {
            let safe = state.get_position();
            if self.skip_whitespace(state) {
                continue;
            }
            if self.lex_number(state) {
                continue;
            }
            if self.lex_string(state) {
                continue;
            }
            if self.lex_ident(state) {
                continue;
            }
            if self.lex_ops_and_delims(state) {
                continue;
            }
            state.advance_if_dead_lock(safe);
        }
        Ok(())
    }

    fn skip_whitespace<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        ATHENA_WHITESPACE.scan(state, AthenaTokenType::Whitespace)
    }

    fn lex_number<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        let Some(first) = state.peek() else {
            return false;
        };
        if !first.is_ascii_digit() {
            return false;
        }
        let mut is_real = false;
        state.advance(first.len_utf8());
        while let Some(c) = state.peek() {
            if c.is_ascii_digit() {
                state.advance(1);
            } else {
                break;
            }
        }
        if state.peek() == Some('.') {
            if state.peek_next_n(1).map(|c| c.is_ascii_digit()).unwrap_or(false) {
                is_real = true;
                state.advance(1);
                while let Some(c) = state.peek() {
                    if c.is_ascii_digit() {
                        state.advance(1);
                    } else {
                        break;
                    }
                }
            }
        }
        let end = state.get_position();
        state.add_token(
            if is_real {
                AthenaTokenType::Real
            } else {
                AthenaTokenType::Integer
            },
            start,
            end,
        );
        true
    }

    fn lex_string<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        if state.peek() != Some('"') {
            return false;
        }
        state.advance(1);
        while let Some(c) = state.peek() {
            if c == '"' {
                state.advance(1);
                break;
            }
            if c == '\\' {
                state.advance(1);
                if state.peek().is_some() {
                    state.advance(1);
                }
                continue;
            }
            state.advance(c.len_utf8());
        }
        state.add_token(AthenaTokenType::String, start, state.get_position());
        true
    }

    fn lex_ident<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        let Some(first) = state.peek() else {
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == '_') {
            return false;
        }
        state.advance(first.len_utf8());
        while let Some(c) = state.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                state.advance(c.len_utf8());
            } else {
                break;
            }
        }
        state.add_token(AthenaTokenType::Identifier, start, state.get_position());
        true
    }

    fn lex_ops_and_delims<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        let Some(c) = state.peek() else {
            return false;
        };
        let kind = match c {
            '+' => AthenaTokenType::Plus,
            '-' => AthenaTokenType::Minus,
            '*' => AthenaTokenType::Times,
            '/' => AthenaTokenType::Divide,
            '^' => AthenaTokenType::Power,
            '(' => AthenaTokenType::LeftParen,
            ')' => AthenaTokenType::RightParen,
            '[' => AthenaTokenType::LeftBracket,
            ']' => AthenaTokenType::RightBracket,
            '{' => AthenaTokenType::LeftBrace,
            '}' => AthenaTokenType::RightBrace,
            ':' => AthenaTokenType::Colon,
            ',' => AthenaTokenType::Comma,
            _ => return false,
        };
        state.advance(c.len_utf8());
        state.add_token(kind, start, state.get_position());
        true
    }
}
