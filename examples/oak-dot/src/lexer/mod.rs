#![doc = include_str!("readme.md")]
/// Token types for the DOT language.
pub mod token_type;

use crate::{language::DotLanguage, lexer::token_type::DotTokenType};
use oak_core::{Lexer, LexerCache, LexerState, OakError, lexer::LexOutput, source::Source};

pub(crate) type State<'a, S> = LexerState<'a, S, DotLanguage>;

/// Lexical analyzer for the DOT language.
#[derive(Clone)]
pub struct DotLexer<'config> {
    config: &'config DotLanguage,
}

impl<'config> DotLexer<'config> {
    /// Creates a new DOT lexer with the given configuration.
    pub fn new(config: &'config DotLanguage) -> Self {
        Self { config }
    }

    /// Skips whitespace characters.
    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
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
            state.add_token(DotTokenType::Whitespace, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles newlines.
    fn lex_newline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('\n') = state.peek() {
            state.advance(1);
            state.add_token(DotTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            state.add_token(DotTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles comments.
    fn lex_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if state.consume_if_starts_with("//") {
            // Single-line comment
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8());
            }

            state.add_token(DotTokenType::Comment, start_pos, state.get_position());
            true
        }
        else if state.consume_if_starts_with("/*") {
            // Multi-line comment
            while let Some(ch) = state.peek() {
                if ch == '*' && state.peek_next_n(1) == Some('/') {
                    state.advance(2); // Skip */
                    break;
                }
                state.advance(ch.len_utf8());
            }

            state.add_token(DotTokenType::Comment, start_pos, state.get_position());
            true
        }
        else if state.consume_if_starts_with("#") {
            // # style comment
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8());
            }

            state.add_token(DotTokenType::Comment, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles identifiers or keywords.
    fn lex_identifier_or_keyword<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            if ch.is_alphabetic() || ch == '_' {
                state.advance(ch.len_utf8());

                while let Some(ch) = state.peek() {
                    if ch.is_alphanumeric() || ch == '_' {
                        state.advance(ch.len_utf8());
                    }
                    else {
                        break;
                    }
                }

                let end_pos = state.get_position();
                let text = state.get_text_in((start_pos..end_pos).into());

                let token_kind = match text.to_lowercase().as_str() {
                    "graph" => DotTokenType::Graph,
                    "digraph" => DotTokenType::Digraph,
                    "subgraph" => DotTokenType::Subgraph,
                    "node" => DotTokenType::Node,
                    "edge" => DotTokenType::Edge,
                    "strict" => DotTokenType::Strict,
                    _ => DotTokenType::Identifier,
                };

                state.add_token(token_kind, start_pos, state.get_position());
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }

    /// Handles numbers.
    fn lex_number<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let is_negative = ch == '-';
            let mut has_digit = false;

            if is_negative {
                // Check if there is a digit after the negative sign
                if let Some(next_ch) = state.peek_next_n(1) {
                    if next_ch.is_ascii_digit() {
                        state.advance(1); // Skip negative sign
                    }
                    else {
                        return false;
                    }
                }
                else {
                    return false;
                }
            }

            if let Some(ch) = state.peek() {
                if ch.is_ascii_digit() {
                    has_digit = true;
                    state.advance(ch.len_utf8());

                    // Handle integer part
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_digit() {
                            state.advance(ch.len_utf8());
                        }
                        else {
                            break;
                        }
                    }

                    // Handle fractional part
                    if let Some('.') = state.peek() {
                        let dot_pos = state.get_position();
                        state.advance(1);

                        if let Some(ch) = state.peek() {
                            if ch.is_ascii_digit() {
                                while let Some(ch) = state.peek() {
                                    if ch.is_ascii_digit() {
                                        state.advance(ch.len_utf8());
                                    }
                                    else {
                                        break;
                                    }
                                }
                            }
                            else {
                                // Backtrack dot
                                state.set_position(dot_pos);
                            }
                        }
                        else {
                            // Backtrack dot
                            state.set_position(dot_pos);
                        }
                    }
                }
            }

            if has_digit || (is_negative && state.get_position() > start_pos + 1) {
                state.add_token(DotTokenType::Number, start_pos, state.get_position());
                true
            }
            else {
                // Backtrack to start position
                state.set_position(start_pos);
                false
            }
        }
        else {
            false
        }
    }

    /// Handles strings.
    fn lex_string<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('"') = state.peek() {
            state.advance(1);

            while let Some(ch) = state.peek() {
                if ch == '"' {
                    state.advance(1);
                    state.add_token(DotTokenType::String, start_pos, state.get_position());
                    return true;
                }
                else if ch == '\\' {
                    state.advance(1);
                    if state.peek().is_some() {
                        state.advance(1);
                    }
                }
                else {
                    state.advance(ch.len_utf8());
                }
            }

            // Unclosed string
            state.add_token(DotTokenType::Error, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles operators.
    fn lex_operator<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if state.consume_if_starts_with("->") {
            state.add_token(DotTokenType::Arrow, start_pos, state.get_position());
            return true;
        }
        if state.consume_if_starts_with("--") {
            state.add_token(DotTokenType::Line, start_pos, state.get_position());
            return true;
        }

        if let Some(ch) = state.peek() {
            match ch {
                '=' => {
                    state.advance(1);
                    state.add_token(DotTokenType::Equal, start_pos, state.get_position());
                    true
                }
                ';' => {
                    state.advance(1);
                    state.add_token(DotTokenType::Semicolon, start_pos, state.get_position());
                    true
                }
                ',' => {
                    state.advance(1);
                    state.add_token(DotTokenType::Comma, start_pos, state.get_position());
                    true
                }
                ':' => {
                    state.advance(1);
                    state.add_token(DotTokenType::Colon, start_pos, state.get_position());
                    true
                }
                _ => false,
            }
        }
        else {
            false
        }
    }

    /// Handles delimiters.
    fn lex_delimiter<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let token_kind = match ch {
                '{' => DotTokenType::LeftBrace,
                '}' => DotTokenType::RightBrace,
                '[' => DotTokenType::LeftBracket,
                ']' => DotTokenType::RightBracket,
                '(' => DotTokenType::LeftParen,
                ')' => DotTokenType::RightParen,
                _ => return false,
            };

            state.advance(ch.len_utf8());
            state.add_token(token_kind, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }
}

impl<'config> Lexer<DotLanguage> for DotLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[oak_core::source::TextEdit], cache: &'a mut impl LexerCache<DotLanguage>) -> LexOutput<DotLanguage> {
        let mut state = State::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> DotLexer<'config> {
    /// Main lexical analysis logic.
    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            // Try various lexical rules
            if self.skip_whitespace(state) {
                continue;
            }

            if self.lex_newline(state) {
                continue;
            }

            if self.lex_comment(state) {
                continue;
            }

            if self.lex_identifier_or_keyword(state) {
                continue;
            }

            if self.lex_number(state) {
                continue;
            }

            if self.lex_string(state) {
                continue;
            }

            if self.lex_operator(state) {
                continue;
            }

            if self.lex_delimiter(state) {
                continue;
            }

            // If no rules match, skip the current character and mark it as an error
            let start_pos = state.get_position();
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
                state.add_token(DotTokenType::Error, start_pos, state.get_position());
            }

            state.advance_if_dead_lock(safe_point);
        }

        Ok(())
    }
}
