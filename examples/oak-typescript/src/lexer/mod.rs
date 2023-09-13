#![doc = include_str!("readme.md")]
/// Token types for the TypeScript language.
pub mod token_type;

pub use self::token_type::TypeScriptTokenType;
use crate::language::TypeScriptLanguage;
use oak_core::{Lexer, LexerCache, LexerState, OakError, TextEdit, lexer::LexOutput, source::Source};

/// Lexer for the TypeScript language.
#[derive(Clone, Debug)]
pub struct TypeScriptLexer<'config> {
    config: &'config TypeScriptLanguage,
}

pub(crate) type State<'a, S> = LexerState<'a, S, TypeScriptLanguage>;

impl<'config> TypeScriptLexer<'config> {
    /// Creates a new `TypeScriptLexer` with the given language configuration.
    pub fn new(config: &'config TypeScriptLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Lexer<TypeScriptLanguage> for TypeScriptLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, text: &S, edits: &[TextEdit], cache: &'a mut impl LexerCache<TypeScriptLanguage>) -> LexOutput<TypeScriptLanguage> {
        let relex_from = edits.iter().map(|e| e.span.start).min().unwrap_or(0);
        let mut state: State<'_, S> = LexerState::new_with_cache(text, relex_from, cache);

        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof()
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> TypeScriptLexer<'config> {
    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

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

            if self.lex_template_literal(state) {
                continue;
            }

            if self.lex_numeric_literal(state) {
                continue;
            }

            if self.lex_identifier_or_keyword(state) {
                continue;
            }

            if self.lex_operator_or_punctuation(state) {
                continue;
            }

            // If all rules do not match, skip current character and mark as error
            let start_pos = state.get_position();
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
                state.add_token(TypeScriptTokenType::Error, start_pos, state.get_position());
            }

            state.advance_if_dead_lock(safe_point);
        }

        Ok(())
    }

    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let mut found = false;

        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' {
                state.advance(ch.len_utf8());
                found = true;
            }
            else {
                break;
            }
        }

        if found {
            state.add_token(TypeScriptTokenType::Whitespace, start, state.get_position());
        }

        found
    }

    fn lex_newline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if let Some(ch) = state.peek() {
            if ch == '\n' {
                state.advance(1);
                state.add_token(TypeScriptTokenType::Newline, start, state.get_position());
                return true;
            }
            else if ch == '\r' {
                state.advance(1);
                if state.peek() == Some('\n') {
                    state.advance(1);
                }
                state.add_token(TypeScriptTokenType::Newline, start, state.get_position());
                return true;
            }
        }

        false
    }

    fn skip_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let rest = state.rest();

        // Line comment: // ...
        if rest.starts_with("//") {
            state.advance(2);
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8());
            }
            state.add_token(TypeScriptTokenType::LineComment, start, state.get_position());
            return true;
        }

        // Block comment: /* ... */
        if rest.starts_with("/*") {
            state.advance(2);
            while let Some(ch) = state.peek() {
                if ch == '*' && state.peek_next_n(1) == Some('/') {
                    state.advance(2);
                    break;
                }
                state.advance(ch.len_utf8());
            }
            state.add_token(TypeScriptTokenType::BlockComment, start, state.get_position());
            return true;
        }

        false
    }

    fn lex_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if let Some(quote) = state.peek() {
            if quote == '"' || quote == '\'' {
                state.advance(1);

                while let Some(ch) = state.peek() {
                    if ch == quote {
                        state.advance(1);
                        break;
                    }
                    else if ch == '\\' {
                        state.advance(1);
                        if let Some(_) = state.peek() {
                            state.advance(1);
                        }
                    }
                    else {
                        state.advance(ch.len_utf8());
                    }
                }

                state.add_token(TypeScriptTokenType::StringLiteral, start, state.get_position());
                return true;
            }
        }

        false
    }

    fn lex_template_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if state.peek() == Some('`') {
            state.advance(1);

            while let Some(ch) = state.peek() {
                if ch == '`' {
                    state.advance(1);
                    break;
                }
                else if ch == '\\' {
                    state.advance(1);
                    if let Some(_) = state.peek() {
                        state.advance(1);
                    }
                }
                else {
                    state.advance(ch.len_utf8());
                }
            }

            state.add_token(TypeScriptTokenType::TemplateString, start, state.get_position());
            return true;
        }

        false
    }

    fn lex_numeric_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if let Some(ch) = state.peek() {
            if ch.is_ascii_digit() {
                state.advance(1);

                // Handle hexadecimal
                if ch == '0' && (state.peek() == Some('x') || state.peek() == Some('X')) {
                    state.advance(1);
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_hexdigit() {
                            state.advance(1);
                        }
                        else {
                            break;
                        }
                    }
                }
                else {
                    // Handle decimal
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_digit() {
                            state.advance(1);
                        }
                        else if ch == '.' && state.peek_next_n(1).map_or(false, |c| c.is_ascii_digit()) {
                            state.advance(1);
                            while let Some(ch) = state.peek() {
                                if ch.is_ascii_digit() {
                                    state.advance(1);
                                }
                                else {
                                    break;
                                }
                            }
                            break;
                        }
                        else {
                            break;
                        }
                    }
                }

                // Check BigInt suffix
                if state.peek() == Some('n') {
                    state.advance(1);
                    state.add_token(TypeScriptTokenType::BigIntLiteral, start, state.get_position());
                }
                else {
                    state.add_token(TypeScriptTokenType::NumericLiteral, start, state.get_position());
                }

                return true;
            }
        }

        false
    }

    fn lex_identifier_or_keyword<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        if let Some(ch) = state.peek() {
            // Private identifiers (`#load`) as a single IdentifierName token including `#`.
            let private = ch == '#';
            if private {
                state.advance(1);
                let Some(next) = state.peek() else {
                    return false;
                };
                if !(next.is_alphabetic() || next == '_' || next == '$') {
                    // Lone `#` is not an identifier; back out so other lexers can try.
                    state.set_position(start);
                    return false;
                }
            }
            else if !(ch.is_alphabetic() || ch == '_' || ch == '$') {
                return false;
            }

            if !private {
                state.advance(ch.len_utf8());
            }
            else {
                // Consume the first identifier char after `#`.
                if let Some(next) = state.peek() {
                    state.advance(next.len_utf8());
                }
            }

            while let Some(ch) = state.peek() {
                if ch.is_alphanumeric() || ch == '_' || ch == '$' {
                    state.advance(ch.len_utf8());
                }
                else {
                    break;
                }
            }

            // Get identifier text and check if it's a keyword (private names stay IdentifierName).
            let end = state.get_position();
            let text = state.get_text_in(oak_core::Range { start, end });
            let kind = if private {
                TypeScriptTokenType::IdentifierName
            }
            else {
                self.keyword_or_identifier(&text)
            };

            state.add_token(kind, start, state.get_position());
            return true;
        }

        false
    }

    fn keyword_or_identifier(&self, text: &str) -> TypeScriptTokenType {
        TypeScriptTokenType::from_keyword(text).unwrap_or(TypeScriptTokenType::IdentifierName)
    }

    fn lex_operator_or_punctuation<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let rest = state.rest();

        let ops = [
            ("===", TypeScriptTokenType::EqualEqualEqual),
            ("!==", TypeScriptTokenType::NotEqualEqual),
            (">>>", TypeScriptTokenType::UnsignedRightShift),
            ("...", TypeScriptTokenType::DotDotDot),
            ("**=", TypeScriptTokenType::StarStarEqual),
            ("<<=", TypeScriptTokenType::LeftShiftEqual),
            (">>=", TypeScriptTokenType::RightShiftEqual),
            ("&&=", TypeScriptTokenType::AmpersandAmpersandEqual),
            ("||=", TypeScriptTokenType::PipePipeEqual),
            ("??=", TypeScriptTokenType::QuestionQuestionEqual),
            ("**", TypeScriptTokenType::StarStar),
            ("<=", TypeScriptTokenType::LessEqual),
            (">=", TypeScriptTokenType::GreaterEqual),
            ("==", TypeScriptTokenType::EqualEqual),
            ("!=", TypeScriptTokenType::NotEqual),
            ("&&", TypeScriptTokenType::AmpersandAmpersand),
            ("||", TypeScriptTokenType::PipePipe),
            ("<<", TypeScriptTokenType::LeftShift),
            (">>", TypeScriptTokenType::RightShift),
            ("++", TypeScriptTokenType::PlusPlus),
            ("--", TypeScriptTokenType::MinusMinus),
            ("=>", TypeScriptTokenType::Arrow),
            ("?.", TypeScriptTokenType::QuestionDot),
            ("??", TypeScriptTokenType::QuestionQuestion),
            ("+=", TypeScriptTokenType::PlusEqual),
            ("-=", TypeScriptTokenType::MinusEqual),
            ("*=", TypeScriptTokenType::StarEqual),
            ("/=", TypeScriptTokenType::SlashEqual),
            ("%=", TypeScriptTokenType::PercentEqual),
            ("&=", TypeScriptTokenType::AmpersandEqual),
            ("|=", TypeScriptTokenType::PipeEqual),
            ("^=", TypeScriptTokenType::CaretEqual),
        ];

        for (op, kind) in ops {
            if rest.starts_with(op) {
                state.advance(op.len());
                state.add_token(kind, start, state.get_position());
                return true;
            }
        }

        if let Some(ch) = state.peek() {
            let kind = match ch {
                '+' => TypeScriptTokenType::Plus,
                '-' => TypeScriptTokenType::Minus,
                '*' => TypeScriptTokenType::Star,
                '/' => TypeScriptTokenType::Slash,
                '%' => TypeScriptTokenType::Percent,
                '<' => TypeScriptTokenType::Less,
                '>' => TypeScriptTokenType::Greater,
                '!' => TypeScriptTokenType::Exclamation,
                '&' => TypeScriptTokenType::Ampersand,
                '|' => TypeScriptTokenType::Pipe,
                '^' => TypeScriptTokenType::Caret,
                '~' => TypeScriptTokenType::Tilde,
                '=' => TypeScriptTokenType::Equal,
                '?' => TypeScriptTokenType::Question,
                '(' => TypeScriptTokenType::LeftParen,
                ')' => TypeScriptTokenType::RightParen,
                '{' => TypeScriptTokenType::LeftBrace,
                '}' => TypeScriptTokenType::RightBrace,
                '[' => TypeScriptTokenType::LeftBracket,
                ']' => TypeScriptTokenType::RightBracket,
                ';' => TypeScriptTokenType::Semicolon,
                ',' => TypeScriptTokenType::Comma,
                '.' => TypeScriptTokenType::Dot,
                ':' => TypeScriptTokenType::Colon,
                '@' => TypeScriptTokenType::At,
                _ => return false,
            };

            state.advance(1);
            state.add_token(kind, start, state.get_position());
            return true;
        }

        false
    }
}
