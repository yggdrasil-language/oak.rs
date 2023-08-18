#![doc = include_str!("readme.md")]

/// Wolfram token types.
pub mod token_type;

use crate::{language::WolframLanguage, lexer::token_type::WolframTokenType};
use oak_core::{
    Lexer, LexerCache, LexerState, OakError,
    lexer::{CommentConfig, LexOutput, StringConfig, WhitespaceConfig},
    source::{Source, TextEdit},
};
use std::sync::LazyLock;

pub(crate) type State<'a, S> = LexerState<'a, S, WolframLanguage>;

static WL_WHITESPACE: LazyLock<WhitespaceConfig> = LazyLock::new(|| WhitespaceConfig { unicode_whitespace: true });
static WL_COMMENT: LazyLock<CommentConfig> = LazyLock::new(|| CommentConfig { line_marker: "", block_start: "(*", block_end: "*)", nested_blocks: true });
static WL_STRING: LazyLock<StringConfig> = LazyLock::new(|| StringConfig { quotes: &['"'], escape: Some('\\') });

/// A lexer for the Wolfram language.
#[derive(Clone, Debug)]
pub struct WolframLexer<'config> {
    /// The Wolfram language configuration.
    config: &'config WolframLanguage,
}

impl<'config> Lexer<WolframLanguage> for WolframLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<WolframLanguage>) -> LexOutput<WolframLanguage> {
        let mut state = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> WolframLexer<'config> {
    /// Creates a new `WolframLexer` with the given configuration.
    pub fn new(config: &'config WolframLanguage) -> Self {
        Self { config }
    }

    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if self.skip_whitespace(state) {
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

            if self.lex_identifier_or_keyword(state) {
                continue;
            }

            if self.lex_operators(state) {
                continue;
            }

            if self.lex_single_char_tokens(state) {
                continue;
            }

            state.advance_if_dead_lock(safe_point);
        }

        Ok(())
    }

    fn skip_whitespace<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        // Handle newlines first
        if let Some(ch) = state.peek() {
            if ch == '\n' || ch == '\r' {
                let start = state.get_position();
                state.advance(ch.len_utf8());
                if ch == '\r' && state.peek() == Some('\n') {
                    state.advance(1);
                }
                state.add_token(WolframTokenType::Newline, start, state.get_position());
                return true;
            }
        }

        if WL_WHITESPACE.scan(state, WolframTokenType::Whitespace) {
            return true;
        }

        false
    }

    fn skip_comment<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        WL_COMMENT.scan(state, WolframTokenType::Comment, WolframTokenType::Comment)
    }

    fn lex_string_literal<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        WL_STRING.scan(state, WolframTokenType::String)
    }

    fn lex_number_literal<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start = state.get_position();
        let first = match state.peek() {
            Some(c) => c,
            None => return false,
        };

        if !first.is_ascii_digit() {
            return false;
        }

        let mut is_real = false;

        // Integer part
        state.advance(first.len_utf8());
        while let Some(c) = state.peek() {
            if c.is_ascii_digit() {
                state.advance(1);
            }
            else {
                break;
            }
        }

        // Decimal part
        if state.peek() == Some('.') {
            let next = state.peek_next_n(1);
            if next.map(|c| c.is_ascii_digit()).unwrap_or(false) {
                is_real = true;
                state.advance(1); // consume '.'
                while let Some(c) = state.peek() {
                    if c.is_ascii_digit() {
                        state.advance(1);
                    }
                    else {
                        break;
                    }
                }
            }
        }

        // Scientific notation
        if let Some(c) = state.peek() {
            if c == 'e' || c == 'E' {
                let next = state.peek_next_n(1);
                if next == Some('+') || next == Some('-') || next.map(|d| d.is_ascii_digit()).unwrap_or(false) {
                    is_real = true;
                    state.advance(1);
                    if let Some(sign) = state.peek() {
                        if sign == '+' || sign == '-' {
                            state.advance(1);
                        }
                    }
                    while let Some(d) = state.peek() {
                        if d.is_ascii_digit() {
                            state.advance(1);
                        }
                        else {
                            break;
                        }
                    }
                }
            }
        }

        let end = state.get_position();
        state.add_token(if is_real { WolframTokenType::Real } else { WolframTokenType::Integer }, start, end);
        true
    }

    fn lex_identifier_or_keyword<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start = state.get_position();
        let ch = match state.peek() {
            Some(c) => c,
            None => return false,
        };

        if !(ch.is_ascii_alphabetic() || ch == '$') {
            return false;
        }

        state.advance(ch.len_utf8());
        while let Some(c) = state.peek() {
            if c.is_ascii_alphanumeric() || c == '$' {
                state.advance(c.len_utf8());
            }
            else {
                break;
            }
        }

        let end = state.get_position();
        let text = state.source().get_text_in((start..end).into());
        let kind = match text.as_ref() {
            "If" => WolframTokenType::If,
            "Then" => WolframTokenType::Then,
            "Else" => WolframTokenType::Else,
            "While" => WolframTokenType::While,
            "For" => WolframTokenType::For,
            "Do" => WolframTokenType::Do,
            "Function" => WolframTokenType::Function,
            "Module" => WolframTokenType::Module,
            "Block" => WolframTokenType::Block,
            "With" => WolframTokenType::With,
            "Table" => WolframTokenType::Table,
            "Map" => WolframTokenType::Map,
            "Apply" => WolframTokenType::Apply,
            "Select" => WolframTokenType::Select,
            "Cases" => WolframTokenType::Cases,
            "Rule" => WolframTokenType::Rule,
            "RuleDelayed" => WolframTokenType::RuleDelayed,
            "Set" => WolframTokenType::Set,
            "SetDelayed" => WolframTokenType::SetDelayed,
            "Unset" => WolframTokenType::Unset,
            "Clear" => WolframTokenType::Clear,
            "ClearAll" => WolframTokenType::ClearAll,
            "Return" => WolframTokenType::Return,
            "Break" => WolframTokenType::Break,
            "Continue" => WolframTokenType::Continue,
            "True" => WolframTokenType::True,
            "False" => WolframTokenType::False,
            "Null" => WolframTokenType::Null,
            "Export" => WolframTokenType::Export,
            "Import" => WolframTokenType::Import,
            _ => WolframTokenType::Identifier,
        };
        state.add_token(kind, start, end);
        true
    }

    fn lex_operators<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start = state.get_position();

        // Multi-character operators (prefer longest matches first)
        let patterns: &[(&str, WolframTokenType)] = &[
            ("===", WolframTokenType::Equal),    // SameQ
            ("=!=", WolframTokenType::NotEqual), // UnsameQ
            ("@@@", WolframTokenType::ApplyLevelOperator),
            ("//@", WolframTokenType::MapAllOperator),
            (":=", WolframTokenType::SetDelayed),
            (":>", WolframTokenType::RuleDelayedOp),
            ("->", WolframTokenType::Arrow),
            ("=>", WolframTokenType::DoubleArrow),
            ("/@", WolframTokenType::MapOperator),
            ("@@", WolframTokenType::ApplyOperator),
            ("//", WolframTokenType::SlashSlash),
            ("@*", WolframTokenType::AtStar),
            ("/*", WolframTokenType::StarSlash),
            ("<>", WolframTokenType::StringJoin),
            ("==", WolframTokenType::Equal),
            ("!=", WolframTokenType::NotEqual),
            ("<=", WolframTokenType::LessEqual),
            (">=", WolframTokenType::GreaterEqual),
            ("&&", WolframTokenType::And),
            ("||", WolframTokenType::Or),
            ("+=", WolframTokenType::AddTo),
            ("-=", WolframTokenType::SubtractFrom),
            ("*=", WolframTokenType::TimesBy),
            ("/=", WolframTokenType::DivideBy),
            ("!!", WolframTokenType::Factorial), // Double Factorial
            ("___", WolframTokenType::TripleUnderscore),
            ("__", WolframTokenType::DoubleUnderscore),
            ("::", WolframTokenType::MessageName),
            ("??", WolframTokenType::DoubleQuestion),
        ];

        for (pat, kind) in patterns {
            if state.starts_with(pat) {
                state.advance(pat.len());
                state.add_token(*kind, start, state.get_position());
                return true;
            }
        }

        // `#` / `#n` Slot and `##` / `##n` SlotSequence (digits stay in the same token).
        if state.starts_with("##") {
            state.advance(2);
            while state.peek().is_some_and(|c| c.is_ascii_digit()) {
                state.advance(1);
            }
            state.add_token(WolframTokenType::SlotSequence, start, state.get_position());
            return true;
        }
        if state.starts_with("#") {
            state.advance(1);
            while state.peek().is_some_and(|c| c.is_ascii_digit()) {
                state.advance(1);
            }
            state.add_token(WolframTokenType::Slot, start, state.get_position());
            return true;
        }

        // Single-character operators
        if let Some(ch) = state.peek() {
            let kind = match ch {
                '+' => Some(WolframTokenType::Plus),
                '-' => Some(WolframTokenType::Minus),
                '*' => Some(WolframTokenType::Times),
                '/' => Some(WolframTokenType::Divide),
                '^' => Some(WolframTokenType::Power),
                '=' => Some(WolframTokenType::Assign),
                '<' => Some(WolframTokenType::Less),
                '>' => Some(WolframTokenType::Greater),
                '?' => Some(WolframTokenType::Question),
                '_' => Some(WolframTokenType::Underscore),
                '.' => Some(WolframTokenType::Dot),
                ':' => Some(WolframTokenType::Colon),
                '@' => Some(WolframTokenType::At),
                '&' => Some(WolframTokenType::Ampersand),
                '!' => Some(WolframTokenType::Factorial),
                '\'' => Some(WolframTokenType::Prime),
                _ => None,
            };
            if let Some(k) = kind {
                state.advance(ch.len_utf8());
                state.add_token(k, start, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_single_char_tokens<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.peek() {
            let kind = match ch {
                '(' => WolframTokenType::LeftParen,
                ')' => WolframTokenType::RightParen,
                '[' => WolframTokenType::LeftBracket,
                ']' => WolframTokenType::RightBracket,
                '{' => WolframTokenType::LeftBrace,
                '}' => WolframTokenType::RightBrace,
                ',' => WolframTokenType::Comma,
                ';' => WolframTokenType::Semicolon,
                _ => {
                    // Unknown character, treat as error
                    state.advance(ch.len_utf8());
                    state.add_token(WolframTokenType::Error, start, state.get_position());
                    return true;
                }
            };
            state.advance(ch.len_utf8());
            state.add_token(kind, start, state.get_position());
            true
        }
        else {
            false
        }
    }
}
