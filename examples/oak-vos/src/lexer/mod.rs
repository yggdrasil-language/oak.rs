use oak_core::{Lexer, LexerState, Source, TextEdit, lexer::LexOutput};

use crate::language::VosLanguage;

/// Token kinds and source spans.
pub mod token_type;
pub use token_type::{VosToken, VosTokenType};

type State<'a, S> = LexerState<'a, S, VosLanguage>;

/// Oak lexer for the VOS surface syntax.
#[derive(Clone, Debug, Default)]
pub struct VosLexer;

impl VosLexer {
    fn scan_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let start = state.get_position();
        while let Some(ch) = state.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }
        let kind = match state.get_text_in((start..state.get_position()).into()).as_ref() {
            "namespace" => VosTokenType::Namespace,
            "table" => VosTokenType::Table,
            "class" => VosTokenType::Class,
            "enums" => VosTokenType::Enums,
            "flags" => VosTokenType::Flags,
            "obsolete" => VosTokenType::Obsolete,
            "using" => VosTokenType::Using,
            "const" => VosTokenType::Const,
            "service" => VosTokenType::Service,
            "micro" => VosTokenType::Micro,
            "macro" => VosTokenType::Macro,
            "query" => VosTokenType::Query,
            "udf" => VosTokenType::Udf,
            "let" => VosTokenType::Let,
            "return" => VosTokenType::Return,
            "true" | "false" => VosTokenType::BooleanLiteral,
            "null" => VosTokenType::NullLiteral,
            _ => VosTokenType::Identifier,
        };
        state.add_token(kind, start, state.get_position());
    }

    fn scan_string<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let start = state.get_position();
        let quote = state.peek().expect("string starts with a quote");
        state.advance(quote.len_utf8());
        while let Some(ch) = state.peek() {
            state.advance(ch.len_utf8());
            if ch == '\\' {
                if let Some(escaped) = state.peek() {
                    state.advance(escaped.len_utf8());
                }
            }
            else if ch == quote {
                state.add_token(VosTokenType::StringLiteral, start, state.get_position());
                return;
            }
        }
        state.add_error(oak_core::OakError::unexpected_eof(state.get_position(), None));
        state.add_token(VosTokenType::StringLiteral, start, state.get_position());
    }
}

impl Lexer<VosLanguage> for VosLexer {
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[TextEdit], _cache: &'a mut impl oak_core::LexerCache<VosLanguage>) -> LexOutput<VosLanguage> {
        let mut state = State::new(source);
        while state.not_at_end() {
            let start = state.get_position();
            let Some(ch) = state.peek()
            else {
                break;
            };
            if ch.is_whitespace() {
                state.advance(ch.len_utf8());
                while let Some(next) = state.peek() {
                    if next.is_whitespace() {
                        state.advance(next.len_utf8());
                    }
                    else {
                        break;
                    }
                }
                state.add_token(VosTokenType::Whitespace, start, state.get_position());
            }
            else if ch == '#' || (ch == '/' && state.peek_next_n(1) == Some('/')) {
                state.advance(ch.len_utf8());
                if ch == '/' {
                    state.advance(1);
                }
                while let Some(next) = state.peek() {
                    if next == '\n' || next == '\r' {
                        break;
                    }
                    state.advance(next.len_utf8());
                }
                state.add_token(VosTokenType::Comment, start, state.get_position());
            }
            else if ch.is_alphabetic() || ch == '_' {
                self.scan_identifier(&mut state);
            }
            else if ch.is_ascii_digit() {
                state.advance(1);
                while let Some(next) = state.peek() {
                    if next.is_ascii_digit() || next == '.' { state.advance(1) } else { break }
                }
                state.add_token(VosTokenType::NumberLiteral, start, state.get_position());
            }
            else if ch == '"' || ch == '\'' {
                self.scan_string(&mut state);
            }
            else {
                state.advance(ch.len_utf8());
                let kind = match ch {
                    '{' => VosTokenType::LeftBrace,
                    '}' => VosTokenType::RightBrace,
                    '(' => VosTokenType::LeftParen,
                    ')' => VosTokenType::RightParen,
                    '[' => VosTokenType::LeftBracket,
                    ']' => VosTokenType::RightBracket,
                    ':' => VosTokenType::Colon,
                    ';' => VosTokenType::Semicolon,
                    ',' => VosTokenType::Comma,
                    '=' => VosTokenType::Equal,
                    '?' => VosTokenType::Question,
                    '<' => VosTokenType::Less,
                    '>' => VosTokenType::Greater,
                    '.' => VosTokenType::Dot,
                    '@' | '&' | '+' | '-' | '*' | '/' | '!' | '|' | '%' | '^' => VosTokenType::Operator,
                    _ => VosTokenType::Error,
                };
                if kind == VosTokenType::Error {
                    state.add_error(oak_core::OakError::expected_token("valid VOS character", start, None));
                }
                state.add_token(kind, start, state.get_position());
            }
        }
        state.add_eof();
        state.finish(Ok(()))
    }
}
