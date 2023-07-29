#![doc = include_str!("readme.md")]
use oak_core::{
    Lexer, LexerCache, LexerState, OakError, Source,
    lexer::{CommentConfig, LexOutput, StringConfig, WhitespaceConfig},
};
pub mod token_type;

use crate::{language::IniLanguage, lexer::token_type::IniTokenType};

pub(crate) type State<'a, S> = LexerState<'a, S, IniLanguage>;

static _INI_WHITESPACE: WhitespaceConfig = WhitespaceConfig { unicode_whitespace: true };
static _INI_COMMENT: CommentConfig = CommentConfig { line_marker: ";", block_start: "", block_end: "", nested_blocks: false };
static _INI_STRING: StringConfig = StringConfig { quotes: &['"', '\''], escape: Some('\\') };

/// INI lexer implementation.
#[derive(Clone, Debug)]
pub struct IniLexer<'config> {
    /// The INI language configuration.
    config: &'config IniLanguage,
}

impl<'config> Lexer<IniLanguage> for IniLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[oak_core::TextEdit], cache: &'a mut impl LexerCache<IniLanguage>) -> LexOutput<IniLanguage> {
        let mut state: State<'_, S> = State::new(source);
        let mut pending_line_value = false;
        let result = self.run(&mut state, &mut pending_line_value);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> IniLexer<'config> {
    /// Creates a new `IniLexer` with the given configuration.
    pub fn new(config: &'config IniLanguage) -> Self {
        Self { config }
    }

    /// The main lexical analysis loop.
    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, pending_line_value: &mut bool) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if *pending_line_value {
                self.lex_line_remainder_value(state);
                *pending_line_value = false;
                continue;
            }

            if self.skip_whitespace(state) {
                continue;
            }

            if self.lex_newline(state) {
                continue;
            }

            if self.skip_comment(state) {
                continue;
            }

            if self.lex_string_literal(state) {
                continue;
            }

            if self.lex_number_literal(state) {
                continue;
            }

            if self.lex_identifier(state) {
                continue;
            }

            if self.lex_punctuation(state, pending_line_value) {
                continue;
            }

            state.advance_if_dead_lock(safe_point);
        }

        Ok(())
    }

    /// After `=` in line-remainder dialect: spaces, then one value token until end of line.
    fn lex_line_remainder_value<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        // Leading spaces/tabs on the value (not newlines).
        let ws_start = state.get_position();
        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }
        if state.get_position() > ws_start {
            state.add_token(IniTokenType::Whitespace, ws_start, state.get_position());
        }

        let start = state.get_position();
        while let Some(ch) = state.peek() {
            if ch == '\n' {
                break;
            }
            // Inline `;` comment ends the value (Westwood line comment).
            if ch == ';' {
                break;
            }
            state.advance(ch.len_utf8());
        }
        // Always emit a value token (may be empty for `Key=`).
        state.add_token(IniTokenType::String, start, state.get_position());
    }

    /// Skips whitespace characters (excluding newlines).
    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' || ch == '\r' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }

        if state.get_position() > start {
            state.add_token(IniTokenType::Whitespace, start, state.get_position());
            return true;
        }
        false
    }

    /// Handles newline characters.
    fn lex_newline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if state.current() == Some('\n') {
            state.advance(1);
            state.add_token(IniTokenType::Newline, start, state.get_position());
            return true;
        }
        false
    }

    /// Skips comments.
    fn skip_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if let Some(ch) = state.current() {
            let is_comment = ch == ';' || (ch == '#' && self.config.hash_comments);
            if is_comment {
                // Skip comment character
                state.advance(1);

                // Read until end of line
                while let Some(ch) = state.peek() {
                    if ch != '\n' {
                        state.advance(ch.len_utf8());
                    }
                    else {
                        break;
                    }
                }

                state.add_token(IniTokenType::Comment, start, state.get_position());
                return true;
            }
        }
        false
    }

    /// Handles string literals.
    fn lex_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if let Some(quote_char) = state.current() {
            if quote_char == '"' || quote_char == '\'' {
                // Skip opening quote
                state.advance(1);

                while let Some(ch) = state.peek() {
                    if ch != quote_char {
                        if ch == '\\' {
                            state.advance(1); // Escape character
                            if let Some(_) = state.peek() {
                                state.advance(1); // Escaped character
                            }
                        }
                        else {
                            state.advance(ch.len_utf8());
                        }
                    }
                    else {
                        // Found closing quote
                        state.advance(1);
                        break;
                    }
                }

                state.add_token(IniTokenType::String, start, state.get_position());
                return true;
            }
        }
        false
    }

    /// Handles number literals.
    fn lex_number_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let first = match state.current() {
            Some(c) => c,
            None => return false,
        };

        // Check if starts with a digit or a sign
        if !first.is_ascii_digit() && first != '-' && first != '+' {
            return false;
        }

        // Westwood type ids like `90mm` must be one name token, not Integer+Identifier.
        if self.config.numeric_keys && first.is_ascii_digit() {
            let mut i = 1usize;
            while let Some(ch) = state.peek_next_n(i) {
                if ch.is_ascii_digit() || ch == '.' || ch == 'e' || ch == 'E' || ch == '+' || ch == '-' {
                    i += 1;
                    continue;
                }
                if ch.is_ascii_alphabetic() || ch == '_' {
                    return false;
                }
                break;
            }
        }

        // If it's a sign, check if followed by a digit
        if first == '-' || first == '+' {
            if let Some(next) = state.peek_next_n(1) {
                if !next.is_ascii_digit() {
                    return false;
                }
            }
            else {
                return false;
            }
        }

        state.advance(1);
        let mut has_dot = false;
        let mut has_exp = false;

        while let Some(ch) = state.peek() {
            if ch.is_ascii_digit() {
                state.advance(1);
            }
            else if ch == '.' && !has_dot && !has_exp {
                has_dot = true;
                state.advance(1);
            }
            else if (ch == 'e' || ch == 'E') && !has_exp {
                has_exp = true;
                state.advance(1);
                // Handle exponent sign
                if let Some(sign) = state.peek() {
                    if sign == '+' || sign == '-' {
                        state.advance(1);
                    }
                }
            }
            else {
                break;
            }
        }

        // Check if it's a valid number
        let end = state.get_position();
        let text = state.get_text_in((start..end).into());

        // Simple validation: cannot be just a sign or just a dot
        if text.as_ref() == "-" || text.as_ref() == "+" || text.as_ref() == "." {
            // Backtrack
            state.set_position(start);
            return false;
        }

        // Determine if it's an integer or a float
        let kind = if has_dot || has_exp { IniTokenType::Float } else { IniTokenType::Integer };

        state.add_token(kind, start, state.get_position());
        true
    }

    /// Handles identifiers
    fn lex_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let ch = match state.current() {
            Some(c) => c,
            None => return false,
        };

        // Identifiers must start with a letter or underscore.
        // With `numeric_keys`, also allow digit-leading names such as `90mm`.
        let digit_ok = self.config.numeric_keys && ch.is_ascii_digit();
        if !(ch.is_ascii_alphabetic() || ch == '_' || digit_ok) {
            return false;
        }

        state.advance(1);
        while let Some(c) = state.current() {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                state.advance(1);
            }
            else {
                break;
            }
        }

        let end = state.get_position();
        let text = state.get_text_in((start..end).into());

        // Pure digits still prefer Integer (lex_number runs first). Digit+letter → Identifier.
        if text.chars().all(|c| c.is_ascii_digit()) {
            state.set_position(start);
            return false;
        }

        // Check if it's a boolean or date-time
        let kind = match text.to_lowercase().as_str() {
            "true" | "false" => IniTokenType::Boolean,
            _ => {
                if self.is_datetime_like(text.as_ref()) {
                    IniTokenType::DateTime
                }
                else {
                    IniTokenType::Identifier
                }
            }
        };

        state.add_token(kind, start, state.get_position());
        true
    }

    /// Handles punctuation
    fn lex_punctuation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, pending_line_value: &mut bool) -> bool {
        use crate::language::IniValueStyle;

        let start = state.get_position();

        // Match longer symbols first
        if state.starts_with("[[") {
            state.advance(2);
            state.add_token(IniTokenType::DoubleLeftBracket, start, state.get_position());
            return true;
        }

        if state.starts_with("]]") {
            state.advance(2);
            state.add_token(IniTokenType::DoubleRightBracket, start, state.get_position());
            return true;
        }

        if let Some(ch) = state.current() {
            let kind = match ch {
                '{' => IniTokenType::LeftBrace,
                '}' => IniTokenType::RightBrace,
                '[' => IniTokenType::LeftBracket,
                ']' => IniTokenType::RightBracket,
                ',' => IniTokenType::Comma,
                '.' => IniTokenType::Dot,
                '=' => IniTokenType::Equal,
                _ => return false,
            };

            state.advance(ch.len_utf8());
            state.add_token(kind, start, state.get_position());
            if kind == IniTokenType::Equal && self.config.value_style == IniValueStyle::LineRemainder {
                *pending_line_value = true;
            }
            return true;
        }

        false
    }

    fn is_datetime_like(&self, text: &str) -> bool {
        // Minimal judgment: those containing - and : might be date-time
        text.contains('-') && text.contains(':')
    }
}
