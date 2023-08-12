#![doc = include_str!("readme.md")]
/// Token types for the Matlab language.
pub mod token_type;

use crate::{language::MatlabLanguage, lexer::token_type::MatlabTokenType};
use oak_core::{
    Lexer, LexerState, TokenType,
    lexer::{LexOutput, LexerCache},
    source::{Source, TextEdit},
};

type State<'s, S> = LexerState<'s, S, MatlabLanguage>;

/// Lexer for the Matlab language.
#[derive(Clone)]
pub struct MatlabLexer<'config> {
    config: &'config MatlabLanguage,
}

impl<'config> Lexer<MatlabLanguage> for MatlabLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<MatlabLanguage>) -> LexOutput<MatlabLanguage> {
        let mut state: State<'_, S> = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> MatlabLexer<'config> {
    /// Creates a new `MatlabLexer` with the given configuration.
    pub fn new(config: &'config MatlabLanguage) -> Self {
        Self { config }
    }

    fn run<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> Result<(), oak_core::OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if self.skip_whitespace(state) {
                continue;
            }

            if self.lex_newline(state) {
                continue;
            }

            if self.lex_comment(state) {
                continue;
            }

            if self.lex_string(state) {
                continue;
            }

            if self.lex_number(state) {
                continue;
            }

            if self.lex_identifier(state) {
                continue;
            }

            if self.lex_operator(state) {
                continue;
            }

            if self.lex_delimiter(state) {
                continue;
            }

            let start_pos = state.get_position();
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
                state.add_token(MatlabTokenType::Error, start_pos, state.get_position());
            }

            state.advance_if_dead_lock(safe_point);
        }

        Ok(())
    }

    fn skip_whitespace<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }
        if state.get_position() > start_pos {
            state.add_token(MatlabTokenType::Whitespace, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn lex_newline<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        if state.consume_if_starts_with("\n") || state.consume_if_starts_with("\r\n") || state.consume_if_starts_with("\r") {
            state.add_token(MatlabTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn lex_identifier<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        if let Some(ch) = state.peek() {
            if ch.is_ascii_alphabetic() || ch == '_' {
                state.advance(ch.len_utf8());
                state.take_while(|c| c.is_ascii_alphanumeric() || c == '_');

                let text = state.get_text_in((start_pos..state.get_position()).into());
                let token_kind = match text.as_ref() {
                    "function" => MatlabTokenType::Function,
                    "end" => MatlabTokenType::End,
                    "if" => MatlabTokenType::If,
                    "else" => MatlabTokenType::Else,
                    "elseif" => MatlabTokenType::Elseif,
                    "while" => MatlabTokenType::While,
                    "for" => MatlabTokenType::For,
                    "parfor" => MatlabTokenType::Parfor,
                    "spmd" => MatlabTokenType::Spmd,
                    "break" => MatlabTokenType::Break,
                    "continue" => MatlabTokenType::Continue,
                    "return" => MatlabTokenType::Return,
                    "switch" => MatlabTokenType::Switch,
                    "case" => MatlabTokenType::Case,
                    "otherwise" => MatlabTokenType::Otherwise,
                    "try" => MatlabTokenType::Try,
                    "catch" => MatlabTokenType::Catch,
                    "global" => MatlabTokenType::Global,
                    "persistent" => MatlabTokenType::Persistent,
                    "classdef" => MatlabTokenType::Classdef,
                    "properties" => MatlabTokenType::Properties,
                    "methods" => MatlabTokenType::Methods,
                    "events" => MatlabTokenType::Events,
                    _ => MatlabTokenType::Identifier,
                };

                state.add_token(token_kind, start_pos, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_number<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        if let Some(ch) = state.peek() {
            if ch.is_ascii_digit() || (ch == '.' && state.peek_next_n(1).map(|c| c.is_ascii_digit()).unwrap_or(false)) {
                if ch == '.' {
                    state.advance(1);
                }
                state.take_while(|c| c.is_ascii_digit());

                if ch != '.' && state.consume_if_starts_with(".") {
                    state.take_while(|c| c.is_ascii_digit());
                }

                if state.consume_if_starts_with("e") || state.consume_if_starts_with("E") {
                    if let Some(sign) = state.peek() {
                        if sign == '+' || sign == '-' {
                            state.advance(1);
                        }
                    }
                    state.take_while(|c| c.is_ascii_digit());
                }

                if state.consume_if_starts_with("i") || state.consume_if_starts_with("j") {
                    // complex
                }

                state.add_token(MatlabTokenType::Number, start_pos, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_string<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        if let Some(quote) = state.peek() {
            // MATLAB: after an expression primary, `'` is transpose (handled by `lex_operator`),
            // not the start of a character vector.
            if quote == '\'' && Self::apostrophe_is_transpose(state) {
                return false;
            }
            if quote == '\'' || quote == '"' {
                state.advance(1);
                while let Some(ch) = state.peek() {
                    if ch == quote {
                        state.advance(1);
                        if state.peek() == Some(quote) {
                            state.advance(1);
                            continue;
                        }
                        break;
                    }
                    else if ch == '\\' {
                        state.advance(1);
                        if let Some(next) = state.peek() {
                            state.advance(next.len_utf8());
                        }
                    }
                    else {
                        state.advance(ch.len_utf8());
                    }
                }
                let kind = if quote == '\'' { MatlabTokenType::Character } else { MatlabTokenType::String };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }
        false
    }

    /// `'` is transpose when the preceding non-trivia token ends an expression primary.
    fn apostrophe_is_transpose(state: &State<'_, impl Source + ?Sized>) -> bool {
        let prev = state.get_tokens().iter().rev().find(|t| !t.kind.is_ignored()).map(|t| t.kind);
        matches!(prev, Some(MatlabTokenType::Identifier | MatlabTokenType::Number | MatlabTokenType::RightParen | MatlabTokenType::RightBracket | MatlabTokenType::RightBrace | MatlabTokenType::Transpose | MatlabTokenType::DotTranspose))
    }

    fn lex_comment<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        if state.consume_if_starts_with("%") {
            if state.consume_if_starts_with("{") {
                let mut depth = 1;
                while depth > 0 && state.not_at_end() {
                    if state.starts_with("%{") {
                        depth += 1;
                        state.advance(2);
                    }
                    else if state.starts_with("%}") {
                        depth -= 1;
                        state.advance(2);
                    }
                    else if let Some(ch) = state.current() {
                        state.advance(ch.len_utf8());
                    }
                }
                state.add_token(MatlabTokenType::BlockComment, start_pos, state.get_position());
            }
            else {
                state.take_while(|c| c != '\n' && c != '\r');
                state.add_token(MatlabTokenType::Comment, start_pos, state.get_position());
            }
            return true;
        }
        false
    }

    fn lex_operator<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        let patterns: &[(&str, MatlabTokenType)] = &[
            (".*", MatlabTokenType::DotTimes),
            ("./", MatlabTokenType::DotDivide),
            (".^", MatlabTokenType::DotPower),
            (".\\", MatlabTokenType::DotLeftDivide),
            (".'", MatlabTokenType::DotTranspose),
            ("==", MatlabTokenType::Equal),
            ("~=", MatlabTokenType::NotEqual),
            ("<=", MatlabTokenType::LessEqual),
            (">=", MatlabTokenType::GreaterEqual),
            ("&&", MatlabTokenType::AndAnd),
            ("||", MatlabTokenType::OrOr),
        ];
        for (pat, kind) in patterns {
            if state.starts_with(pat) {
                state.advance(pat.len());
                state.add_token(*kind, start_pos, state.get_position());
                return true;
            }
        }

        if let Some(ch) = state.peek() {
            let kind = match ch {
                '+' => Some(MatlabTokenType::Plus),
                '-' => Some(MatlabTokenType::Minus),
                '*' => Some(MatlabTokenType::Times),
                '/' => Some(MatlabTokenType::Divide),
                '\\' => Some(MatlabTokenType::LeftDivide),
                '^' => Some(MatlabTokenType::Power),
                '<' => Some(MatlabTokenType::Less),
                '>' => Some(MatlabTokenType::Greater),
                '=' => Some(MatlabTokenType::Assign),
                '~' => Some(MatlabTokenType::Not),
                '&' => Some(MatlabTokenType::And),
                '|' => Some(MatlabTokenType::Or),
                '\'' => Some(MatlabTokenType::Transpose),
                _ => None,
            };
            if let Some(k) = kind {
                state.advance(ch.len_utf8());
                state.add_token(k, start_pos, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_delimiter<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start_pos = state.get_position();
        if let Some(ch) = state.peek() {
            let kind = match ch {
                '(' => MatlabTokenType::LeftParen,
                ')' => MatlabTokenType::RightParen,
                '[' => MatlabTokenType::LeftBracket,
                ']' => MatlabTokenType::RightBracket,
                '{' => MatlabTokenType::LeftBrace,
                '}' => MatlabTokenType::RightBrace,
                ';' => MatlabTokenType::Semicolon,
                ',' => MatlabTokenType::Comma,
                ':' => MatlabTokenType::Colon,
                '?' => MatlabTokenType::Question,
                '@' => MatlabTokenType::At,
                '.' => MatlabTokenType::Dot,
                _ => return false,
            };
            state.advance(ch.len_utf8());
            state.add_token(kind, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }
}
