pub mod token_type;

use crate::{language::MapleLanguage, lexer::token_type::MapleTokenType};
use oak_core::{
    Lexer, LexerState, OakError,
    lexer::{LexOutput, LexerCache},
    source::{Source, TextEdit},
};
type State<'a, S> = LexerState<'a, S, MapleLanguage>;

#[derive(Clone, Debug)]
pub struct MapleLexer<'config> {
    config: &'config MapleLanguage,
}
impl<'config> MapleLexer<'config> {
    pub fn new(config: &'config MapleLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Lexer<MapleLanguage> for MapleLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<MapleLanguage>) -> LexOutput<MapleLanguage> {
        let mut state = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}
impl<'config> MapleLexer<'config> {
    fn run<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> Result<(), OakError> {
        let _ = self.config;
        while state.not_at_end() {
            let start_position = state.get_position();
            if self.whitespace(state) || self.comment(state) || self.number(state) || self.identifier(state) || self.operator(state) {
                continue;
            }
            state.advance_if_dead_lock(start_position);
        }
        Ok(())
    }
    fn whitespace<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        while matches!(state.peek(), Some(' ' | '\t' | '\r' | '\n')) {
            state.advance(1);
        }
        if state.get_position() > start {
            state.add_token(MapleTokenType::Whitespace, start, state.get_position());
            true
        }
        else {
            false
        }
    }
    fn comment<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        if state.peek() != Some('#') {
            return false;
        }
        while let Some(character) = state.peek() {
            state.advance(character.len_utf8());
            if character == '\n' {
                break;
            }
        }
        state.add_token(MapleTokenType::Comment, start, state.get_position());
        true
    }
    fn number<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        if !state.peek().is_some_and(|character| character.is_ascii_digit()) {
            return false;
        }
        while state.peek().is_some_and(|character| character.is_ascii_digit()) {
            state.advance(1);
        }
        let mut float = false;
        if state.peek() == Some('.') && state.peek_next_n(1).is_some_and(|character| character.is_ascii_digit()) {
            float = true;
            state.advance(1);
            while state.peek().is_some_and(|character| character.is_ascii_digit()) {
                state.advance(1);
            }
        }
        state.add_token(if float { MapleTokenType::Float } else { MapleTokenType::Integer }, start, state.get_position());
        true
    }
    fn identifier<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        if !state.peek().is_some_and(|character| character.is_ascii_alphabetic() || character == '_') {
            return false;
        }
        state.advance(1);
        while state.peek().is_some_and(|character| character.is_ascii_alphanumeric() || character == '_') {
            state.advance(1);
        }
        let end = state.get_position();
        let spelling = state.get_text_in(oak_core::Range { start, end });
        let reserved = matches!(
            spelling.as_ref(),
            "and"
                | "assuming"
                | "break"
                | "by"
                | "catch"
                | "description"
                | "do"
                | "done"
                | "elif"
                | "else"
                | "end"
                | "error"
                | "export"
                | "fi"
                | "finally"
                | "for"
                | "from"
                | "global"
                | "if"
                | "implies"
                | "in"
                | "intersect"
                | "local"
                | "minus"
                | "mod"
                | "module"
                | "next"
                | "not"
                | "od"
                | "option"
                | "options"
                | "or"
                | "proc"
                | "quit"
                | "read"
                | "return"
                | "save"
                | "stop"
                | "subset"
                | "then"
                | "to"
                | "try"
                | "union"
                | "until"
                | "use"
                | "uses"
                | "while"
                | "xor"
        );
        state.add_token(if reserved { MapleTokenType::Error } else { MapleTokenType::Identifier }, start, end);
        true
    }
    fn operator<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        let Some(character) = state.peek()
        else {
            return false;
        };
        let kind = match character {
            '+' => MapleTokenType::Plus,
            '-' => MapleTokenType::Minus,
            '*' => MapleTokenType::Times,
            '/' => MapleTokenType::Divide,
            '^' => MapleTokenType::Power,
            '=' => MapleTokenType::Equal,
            '(' => MapleTokenType::LeftParen,
            ')' => MapleTokenType::RightParen,
            '[' => MapleTokenType::LeftBracket,
            ']' => MapleTokenType::RightBracket,
            ',' => MapleTokenType::Comma,
            ';' => MapleTokenType::Semicolon,
            _ => MapleTokenType::Error,
        };
        state.advance(character.len_utf8());
        state.add_token(kind, start, state.get_position());
        true
    }
}
