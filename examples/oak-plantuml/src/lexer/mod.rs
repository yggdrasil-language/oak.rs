//! Line-aware lexer for the PlantUML class and sequence subset.

pub mod token_type;

use crate::{language::PlantUmlLanguage, lexer::token_type::PlantUmlTokenType};
use oak_core::{Lexer, LexerCache, LexerState, TextEdit, errors::OakError, lexer::LexOutput, source::Source};

/// Lexer for the PlantUML class and sequence subset.
pub struct PlantUmlLexer<'config> {
    /// The PlantUML language configuration.
    _config: &'config PlantUmlLanguage,
}

impl<'config> PlantUmlLexer<'config> {
    /// Creates a lexer with the given language configuration.
    pub fn new(config: &'config PlantUmlLanguage) -> Self {
        Self { _config: config }
    }

    fn run<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) -> Result<(), OakError> {
        let mut at_line_start = true;

        while state.not_at_end() {
            let safe_point = state.get_position();
            let Some(ch) = state.peek()
            else {
                break;
            };

            match ch {
                ' ' | '\t' => {
                    self.skip_whitespace(state);
                    continue;
                }
                '\n' | '\r' => {
                    self.lex_newline(state);
                    at_line_start = true;
                    continue;
                }
                '\'' if at_line_start => {
                    self.lex_comment(state);
                    at_line_start = false;
                    continue;
                }
                '@' => {
                    if self.lex_directive(state) {
                        at_line_start = false;
                        continue;
                    }
                    self.lex_text(state);
                }
                '"' => {
                    self.lex_string(state);
                }
                '{' => self.single(state, PlantUmlTokenType::LeftBrace),
                '}' => self.single(state, PlantUmlTokenType::RightBrace),
                '(' => self.single(state, PlantUmlTokenType::LeftParen),
                ')' => self.single(state, PlantUmlTokenType::RightParen),
                '+' => self.single(state, PlantUmlTokenType::Plus),
                '#' => self.single(state, PlantUmlTokenType::Hash),
                '~' => self.single(state, PlantUmlTokenType::Tilde),
                ':' => self.single(state, PlantUmlTokenType::Colon),
                _ => {
                    if self.lex_arrow(state) {
                    }
                    else if ch == '-' {
                        self.single(state, PlantUmlTokenType::Minus);
                    }
                    else if self.lex_keyword(state) {
                    }
                    else if is_identifier_start(ch) {
                        self.lex_identifier(state);
                    }
                    else {
                        self.lex_text(state);
                    }
                }
            }

            if state.get_position() == safe_point {
                let start = state.get_position();
                let Some(ch) = state.peek()
                else {
                    break;
                };
                state.advance(ch.len_utf8());
                state.add_token(PlantUmlTokenType::Error, start, state.get_position());
                let error = state.source().syntax_error("unexpected character in PlantUML input".to_string(), start);
                state.add_error(error);
            }

            at_line_start = false;
        }

        Ok(())
    }

    fn skip_whitespace<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) {
        let start = state.get_position();
        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }
        if state.get_position() > start {
            state.add_token(PlantUmlTokenType::Whitespace, start, state.get_position());
        }
    }

    fn lex_newline<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) {
        let start = state.get_position();
        if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
        }
        else {
            state.advance(1);
        }
        state.add_token(PlantUmlTokenType::Newline, start, state.get_position());
    }

    fn lex_comment<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) {
        let start = state.get_position();
        while let Some(ch) = state.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            state.advance(ch.len_utf8());
        }
        state.add_token(PlantUmlTokenType::Comment, start, state.get_position());
    }

    fn lex_directive<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) -> bool {
        if matches_word(state, "@startuml") {
            let start = state.get_position();
            state.advance("@startuml".len());
            state.add_token(PlantUmlTokenType::StartMarker, start, state.get_position());
            true
        }
        else if matches_word(state, "@enduml") {
            let start = state.get_position();
            state.advance("@enduml".len());
            state.add_token(PlantUmlTokenType::EndMarker, start, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn lex_string<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) {
        let start = state.get_position();
        state.advance(1);
        let mut closed = false;

        while let Some(ch) = state.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            if ch == '\\' {
                state.advance(1);
                if let Some(next) = state.peek() {
                    state.advance(next.len_utf8());
                }
                continue;
            }
            state.advance(ch.len_utf8());
            if ch == '"' {
                closed = true;
                break;
            }
        }

        let kind = if closed { PlantUmlTokenType::String } else { PlantUmlTokenType::Error };
        state.add_token(kind, start, state.get_position());
        if !closed {
            let error = state.source().syntax_error("unterminated PlantUML string".to_string(), start);
            state.add_error(error);
        }
    }

    fn lex_arrow<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) -> bool {
        const ARROWS: &[&str] = &["-->>", "->>", "-->", "->", "<|--", "--|>", "--*", "*--", "--o", "o--", "<..", "..>", "<--", ".."];

        let mut matched = None;
        for arrow in ARROWS {
            if state.starts_with(arrow) {
                matched = Some(*arrow);
                break;
            }
        }

        let Some(arrow) = matched
        else {
            return false;
        };

        let start = state.get_position();
        state.advance(arrow.len());
        state.add_token(PlantUmlTokenType::Arrow, start, state.get_position());
        true
    }

    fn lex_keyword<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) -> bool {
        const KEYWORDS: &[(&str, PlantUmlTokenType)] = &[
            ("deactivate", PlantUmlTokenType::Deactivate),
            ("participant", PlantUmlTokenType::Participant),
            ("abstract", PlantUmlTokenType::Abstract),
            ("activate", PlantUmlTokenType::Activate),
            ("interface", PlantUmlTokenType::Interface),
            ("actor", PlantUmlTokenType::Actor),
            ("class", PlantUmlTokenType::Class),
            ("enum", PlantUmlTokenType::Enum),
            ("as", PlantUmlTokenType::As),
        ];

        for (word, kind) in KEYWORDS {
            if matches_word(state, word) {
                let start = state.get_position();
                state.advance(word.len());
                state.add_token(*kind, start, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_identifier<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) {
        let start = state.get_position();

        while let Some(ch) = state.peek() {
            if is_identifier_continue(ch) {
                state.advance(ch.len_utf8());
            }
            else if ch == '.' && state.peek_next().is_some_and(is_identifier_continue) {
                state.advance(1);
            }
            else {
                break;
            }
        }

        state.add_token(PlantUmlTokenType::Identifier, start, state.get_position());
    }

    fn lex_text<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>) {
        let start = state.get_position();

        while let Some(ch) = state.peek() {
            if ch.is_whitespace() || matches!(ch, '"' | '@' | '{' | '}' | '(' | ')' | '+' | '~' | '#' | ':') {
                break;
            }
            state.advance(ch.len_utf8());
        }

        state.add_token(PlantUmlTokenType::Text, start, state.get_position());
    }

    fn single<S: Source + ?Sized>(&self, state: &mut LexerState<'_, S, PlantUmlLanguage>, kind: PlantUmlTokenType) {
        let start = state.get_position();
        state.advance(1);
        state.add_token(kind, start, state.get_position());
    }
}

impl<'config> Lexer<PlantUmlLanguage> for PlantUmlLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, text: &'a S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<PlantUmlLanguage>) -> LexOutput<PlantUmlLanguage> {
        let mut state = LexerState::new(text);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

/// Returns `true` when the source at the current position starts with the word,
/// compared ASCII-case-insensitively and followed by a non-identifier character.
fn matches_word<S: Source + ?Sized>(state: &mut LexerState<'_, S, PlantUmlLanguage>, word: &str) -> bool {
    let text = state.get_text_from(state.get_position());
    let Some(head) = text.get(..word.len())
    else {
        return false;
    };
    if !head.eq_ignore_ascii_case(word) {
        return false;
    }
    text.get(word.len()..).and_then(|tail| tail.chars().next()).is_none_or(|ch| !is_identifier_continue(ch))
}

fn is_identifier_start(ch: char) -> bool {
    ch.is_alphabetic() || ch == '_' || ch == '$'
}

fn is_identifier_continue(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_' || ch == '$'
}
