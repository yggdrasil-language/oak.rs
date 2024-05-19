#![doc = include_str!("readme.md")]
/// Token type definitions for Typst lexer.
pub mod token_type;

use crate::{language::TypstLanguage, lexer::token_type::TypstTokenType};
use oak_core::{
    Lexer, LexerCache, LexerState, OakError,
    lexer::{CommentConfig, LexOutput, StringConfig, WhitespaceConfig},
    source::{Source, TextEdit},
};
use std::sync::LazyLock;

type State<'s, S> = LexerState<'s, S, TypstLanguage>;

static TYPST_WHITESPACE: LazyLock<WhitespaceConfig> = LazyLock::new(|| WhitespaceConfig { unicode_whitespace: true });
static TYPST_COMMENT: LazyLock<CommentConfig> = LazyLock::new(|| CommentConfig { line_marker: "//", block_start: "/*", block_end: "*/", nested_blocks: true });
static TYPST_STRING: LazyLock<StringConfig> = LazyLock::new(|| StringConfig { quotes: &['"'], escape: Some('\\') });

#[derive(Clone, Debug)]
/// Lexer for Typst source code.
pub struct TypstLexer<'config> {
    config: &'config TypstLanguage,
}

impl<'config> Lexer<TypstLanguage> for TypstLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[TextEdit], _cache: &'a mut impl LexerCache<TypstLanguage>) -> LexOutput<TypstLanguage> {
        let mut state = State::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish(result)
    }
}

impl<'config> TypstLexer<'config> {
    /// Creates a new TypstLexer with the given language configuration.
    pub fn new(config: &'config TypstLanguage) -> Self {
        Self { config }
    }

    fn run<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if self.lex_whitespace(state) {
                continue;
            }

            if TYPST_COMMENT.scan(state, TypstTokenType::LineComment, TypstTokenType::BlockComment) {
                continue;
            }

            if TYPST_STRING.scan(state, TypstTokenType::StringLiteral) {
                continue;
            }

            // 行首标记必须先于数字字面量识别，否则 `1. ` 会被拆成 NumericLiteral 与 Dot，
            // 永远无法产生 EnumItem。
            if self.lex_markup(state) {
                continue;
            }

            if self.lex_number_literal(state) {
                continue;
            }

            if self.lex_identifier_or_keyword(state) {
                continue;
            }

            if self.lex_operators(state) {
                continue;
            }

            if self.lex_single_char_tokens(state) {
                continue;
            }

            if self.lex_text(state) {
                continue;
            }

            state.advance_if_dead_lock(safe_point)
        }

        Ok(())
    }

    fn lex_whitespace<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        if let Some(ch) = state.peek() {
            if ch == '\n' || ch == '\r' {
                let start = state.get_position();
                state.advance(1);
                if ch == '\r' && state.peek() == Some('\n') {
                    state.advance(1);
                }
                state.add_token(TypstTokenType::Newline, start, state.get_position());
                return true;
            }
        }
        TYPST_WHITESPACE.scan(state, TypstTokenType::Whitespace)
    }

    fn lex_number_literal<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        let text = state.rest();
        if text.is_empty() || !text.chars().next().unwrap().is_ascii_digit() {
            return false;
        }

        let mut pos = 0;
        let chars: Vec<char> = text.chars().collect();

        // Integer part
        while pos < chars.len() && chars[pos].is_ascii_digit() {
            pos += 1;
        }

        // Fractional part
        if pos < chars.len() && chars[pos] == '.' && pos + 1 < chars.len() && chars[pos + 1].is_ascii_digit() {
            pos += 1; // Skip '.'
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
        }

        // Exponent part
        if pos < chars.len() && (chars[pos] == 'e' || chars[pos] == 'E') {
            pos += 1;
            if pos < chars.len() && (chars[pos] == '+' || chars[pos] == '-') {
                pos += 1;
            }
            while pos < chars.len() && chars[pos].is_ascii_digit() {
                pos += 1;
            }
        }

        if pos > 0 {
            state.advance(pos);
            state.add_token(TypstTokenType::NumericLiteral, start, state.get_position());
            return true;
        }

        false
    }

    fn lex_markup<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();

        // Check for the beginning of a line
        let is_line_start = start == 0 || matches!(state.source().get_char_at(start - 1), Some('\n') | Some('\r'));

        if let Some(ch) = state.peek() {
            match ch {
                '=' if is_line_start => {
                    let mut count = 0;
                    while state.peek() == Some('=') {
                        count += 1;
                        state.advance(1);
                    }
                    if state.peek() == Some(' ') || state.peek() == Some('\t') {
                        state.add_token(TypstTokenType::Heading, start, state.get_position());
                        return true;
                    }
                }
                '-' | '+' if is_line_start => {
                    state.advance(1);
                    if state.peek() == Some(' ') || state.peek() == Some('\t') {
                        state.add_token(TypstTokenType::ListItem, start, state.get_position());
                        return true;
                    }
                }
                '0'..='9' if is_line_start => {
                    let mut pos = 0;
                    while let Some(c) = state.peek_next_n(pos) {
                        if c.is_ascii_digit() {
                            pos += 1;
                        }
                        else {
                            break;
                        }
                    }
                    if pos > 0 && state.peek_next_n(pos) == Some('.') {
                        pos += 1; // '.'
                        if state.peek_next_n(pos) == Some(' ') || state.peek_next_n(pos) == Some('\t') {
                            state.advance(pos);
                            state.add_token(TypstTokenType::EnumItem, start, state.get_position());
                            return true;
                        }
                    }
                }
                '*' => {
                    let is_escaped = start > 0 && state.source().get_char_at(start - 1) == Some('\\');
                    if !is_escaped {
                        state.advance(1);
                        state.add_token(TypstTokenType::Strong, start, state.get_position());
                        return true;
                    }
                }
                '_' => {
                    let is_escaped = start > 0 && state.source().get_char_at(start - 1) == Some('\\');
                    if !is_escaped {
                        state.advance(1);
                        state.add_token(TypstTokenType::Emphasis, start, state.get_position());
                        return true;
                    }
                }
                _ => {}
            }
        }

        state.set_position(start);
        false
    }

    fn lex_identifier_or_keyword<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        let text = state.rest();
        if text.is_empty() {
            return false;
        }

        let first_char = text.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return false;
        }

        let mut pos = 0;
        let chars: Vec<char> = text.chars().collect();

        // First character
        pos += 1;

        // Subsequent characters
        while pos < chars.len() && (chars[pos].is_ascii_alphanumeric()) {
            pos += 1;
        }

        if pos > 0 {
            let identifier_text = &text[..pos];
            let kind = self.keyword_or_identifier(identifier_text);
            state.advance(pos);
            state.add_token(kind, start, state.get_position());
            return true;
        }

        false
    }

    fn keyword_or_identifier(&self, text: &str) -> TypstTokenType {
        match text {
            "let" => TypstTokenType::Let,
            "if" => TypstTokenType::If,
            "else" => TypstTokenType::Else,
            "for" => TypstTokenType::For,
            "while" => TypstTokenType::While,
            "break" => TypstTokenType::Break,
            "continue" => TypstTokenType::Continue,
            "return" => TypstTokenType::Return,
            "true" => TypstTokenType::True,
            "false" => TypstTokenType::False,
            "set" => TypstTokenType::Set,
            "show" => TypstTokenType::Show,
            "import" => TypstTokenType::Import,
            "include" => TypstTokenType::Include,
            _ => TypstTokenType::Identifier,
        }
    }

    fn lex_operators<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        let text = state.rest();
        if text.is_empty() {
            return false;
        }

        let chars: Vec<char> = text.chars().collect();

        let (kind, len) = match chars[0] {
            '=' => {
                let mut count = 1;
                while count < chars.len() && chars[count] == '=' {
                    count += 1;
                }
                (TypstTokenType::Equal, count)
            }
            '!' => {
                if chars.len() > 1 && chars[1] == '=' {
                    (TypstTokenType::NotEqual, 2)
                }
                else {
                    (TypstTokenType::Not, 1)
                }
            }
            '<' => {
                if chars.len() > 1 && chars[1] == '=' {
                    (TypstTokenType::LessEqual, 2)
                }
                else {
                    (TypstTokenType::Less, 1)
                }
            }
            '>' => {
                if chars.len() > 1 && chars[1] == '=' {
                    (TypstTokenType::GreaterEqual, 2)
                }
                else {
                    (TypstTokenType::Greater, 1)
                }
            }
            '&' => {
                if chars.len() > 1 && chars[1] == '&' {
                    (TypstTokenType::And, 2)
                }
                else {
                    return false;
                }
            }
            '|' => {
                if chars.len() > 1 && chars[1] == '|' {
                    (TypstTokenType::Or, 2)
                }
                else {
                    return false;
                }
            }
            '+' => (TypstTokenType::Plus, 1),
            '-' => (TypstTokenType::Minus, 1),
            '*' => (TypstTokenType::Star, 1),
            '/' => (TypstTokenType::Slash, 1),
            '%' => (TypstTokenType::Percent, 1),
            _ => return false,
        };

        state.advance(len);
        state.add_token(kind, start, state.get_position());
        true
    }

    fn lex_single_char_tokens<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        let text = state.rest();
        if text.is_empty() {
            return false;
        }

        let ch = text.chars().next().unwrap();

        let kind = match ch {
            '(' => TypstTokenType::LeftParen,
            ')' => TypstTokenType::RightParen,
            '{' => TypstTokenType::LeftBrace,
            '}' => TypstTokenType::RightBrace,
            '[' => TypstTokenType::LeftBracket,
            ']' => TypstTokenType::RightBracket,
            ';' => TypstTokenType::Semicolon,
            ',' => TypstTokenType::Comma,
            '.' => TypstTokenType::Dot,
            ':' => TypstTokenType::Colon,
            '#' => TypstTokenType::Hash,
            '@' => TypstTokenType::At,
            '$' => TypstTokenType::Dollar,
            '_' => TypstTokenType::Underscore,
            '`' => TypstTokenType::Backtick,
            _ => return false,
        };

        state.advance(1);
        state.add_token(kind, start, state.get_position());
        true
    }

    fn lex_text<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        let mut has_text = false;

        while let Some(ch) = state.peek() {
            // Text should also consume alphanumeric characters if they don't form identifiers
            // But here we want to break on anything that could be a special token
            if ch.is_whitespace()
                || ch == '/'
                || ch == '"'
                || ch == '='
                || ch == '-'
                || ch == '+'
                || ch == '!'
                || ch == '<'
                || ch == '>'
                || ch == '&'
                || ch == '|'
                || ch == '('
                || ch == ')'
                || ch == '{'
                || ch == '}'
                || ch == '['
                || ch == ']'
                || ch == ';'
                || ch == ','
                || ch == '.'
                || ch == ':'
                || ch == '#'
                || ch == '@'
                || ch == '$'
                || ch == '`'
                || ch == '\\'
            {
                break;
            }

            // Special handling for markup chars that were not handled by lex_markup
            if ch == '*' || ch == '_' {
                break;
            }

            state.advance(ch.len_utf8());
            has_text = true;
        }

        if has_text {
            state.add_token(TypstTokenType::Text, start, state.get_position());
            true
        }
        else {
            false
        }
    }
}
