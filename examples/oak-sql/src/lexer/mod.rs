#![doc = include_str!("readme.md")]
use oak_core::Source;
/// Token types for SQL.
pub mod token_type;
pub use token_type::SqlTokenType;

use crate::language::SqlLanguage;
use oak_core::{
    Lexer, LexerCache, LexerState, OakError, TextEdit,
    lexer::{LexOutput, WhitespaceConfig},
};
use std::sync::LazyLock;

pub(crate) type State<'a, S> = LexerState<'a, S, SqlLanguage>;

static SQL_WHITESPACE: LazyLock<WhitespaceConfig> = LazyLock::new(|| WhitespaceConfig { unicode_whitespace: true });

/// Lexer for SQL.
///
/// This lexer is responsible for breaking down SQL source text into a stream of
/// tokens. it handles different SQL dialects and supports incremental lexing
/// through the [`Lexer`] trait.
///
/// # Supported Features
///
/// - Case-insensitive keywords
/// - Multiple identifier quoting styles (double quotes, backticks, brackets)
/// - Various literal types (strings, numbers, booleans)
/// - Comments (line and block)
#[derive(Clone, Debug)]
pub struct SqlLexer<'config> {
    config: &'config SqlLanguage,
}

impl<'config> Lexer<SqlLanguage> for SqlLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, text: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<SqlLanguage>) -> LexOutput<SqlLanguage> {
        let mut state = State::new(text);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> SqlLexer<'config> {
    /// Creates a new `SqlLexer` with the given configuration.
    pub fn new(config: &'config SqlLanguage) -> Self {
        Self { config }
    }

    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if let Some(ch) = state.peek() {
                match ch {
                    ' ' | '\t' => {
                        self.skip_whitespace(state);
                    }
                    '\n' | '\r' => {
                        self.lex_newline(state);
                    }
                    '-' => {
                        if state.starts_with("--") {
                            self.skip_comment(state);
                        }
                        else {
                            self.lex_operators(state);
                        }
                    }
                    '/' => {
                        if state.starts_with("/*") {
                            self.skip_comment(state);
                        }
                        else {
                            self.lex_operators(state);
                        }
                    }
                    '\'' | '"' => {
                        self.lex_string_literal(state);
                    }
                    '`' if self.config.backtick_identifiers => {
                        self.lex_quoted_identifier(state, '`');
                    }
                    '[' if self.config.bracket_identifiers => {
                        self.lex_bracket_identifier(state);
                    }
                    '0'..='9' => {
                        self.lex_number_literal(state);
                    }
                    'a'..='z' | 'A'..='Z' | '_' => {
                        self.lex_identifier_or_keyword(state);
                    }
                    '<' | '>' | '!' | '=' | '+' | '*' | '%' => {
                        self.lex_operators(state);
                    }
                    '(' | ')' | ',' | ';' | '.' | ':' | '[' | ']' => {
                        self.lex_single_char_tokens(state);
                    }
                    _ => {
                        // If no patterns match, skip current character and add error token
                        state.advance(ch.len_utf8());
                        state.add_token(SqlTokenType::Error, safe_point, state.get_position());
                    }
                }
            }

            state.advance_if_dead_lock(safe_point);
        }
        Ok(())
    }

    /// Handles newlines
    fn lex_newline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('\n') = state.peek() {
            state.advance(1);
            state.add_token(SqlTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            state.add_token(SqlTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        SQL_WHITESPACE.scan(state, SqlTokenType::Whitespace);
        true
    }

    fn skip_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        // Line comment: -- ... until newline
        if state.starts_with("--") {
            state.advance(2);
            state.take_while(|ch| ch != '\n' && ch != '\r');
            state.add_token(SqlTokenType::Comment, start, state.get_position());
            return true;
        }

        // Block comment: /* ... */
        if state.starts_with("/*") {
            state.advance(2);
            while state.not_at_end() {
                if state.starts_with("*/") {
                    state.advance(2);
                    break;
                }
                if let Some(ch) = state.current() {
                    state.advance(ch.len_utf8());
                }
            }
            state.add_token(SqlTokenType::Comment, start, state.get_position());
            return true;
        }

        false
    }

    fn lex_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let quote = match state.current() {
            Some(c) if c == '\'' || c == '"' => {
                state.advance(c.len_utf8());
                c
            }
            _ => return false,
        };

        while let Some(ch) = state.current() {
            if ch == quote {
                state.advance(ch.len_utf8());
                // Handle escaped quotes if necessary (e.g. '' in SQL)
                if state.peek() == Some(quote) {
                    state.advance(quote.len_utf8());
                    continue;
                }
                break;
            }
            state.advance(ch.len_utf8());
        }

        state.add_token(SqlTokenType::StringLiteral, start, state.get_position());
        true
    }

    fn lex_quoted_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, quote: char) -> bool {
        let start = state.get_position();
        state.advance(quote.len_utf8());

        while let Some(ch) = state.current() {
            if ch == quote {
                state.advance(ch.len_utf8());
                break;
            }
            state.advance(ch.len_utf8());
        }

        state.add_token(SqlTokenType::Identifier_, start, state.get_position());
        true
    }

    fn lex_bracket_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        state.advance(1); // '['

        while let Some(ch) = state.current() {
            if ch == ']' {
                state.advance(1);
                break;
            }
            state.advance(ch.len_utf8());
        }

        state.add_token(SqlTokenType::Identifier_, start, state.get_position());
        true
    }

    fn lex_number_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let first = match state.current() {
            Some(c) => c,
            None => return false,
        };

        if !first.is_ascii_digit() {
            return false;
        }

        let mut is_float = false;
        state.advance(1);

        // Integer part
        while let Some(c) = state.peek() {
            if c.is_ascii_digit() || c == '_' {
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
                is_float = true;
                state.advance(1); // consume '.'
                while let Some(c) = state.peek() {
                    if c.is_ascii_digit() || c == '_' {
                        state.advance(1);
                    }
                    else {
                        break;
                    }
                }
            }
        }

        // Exponent part
        if let Some(c) = state.peek() {
            if c == 'e' || c == 'E' {
                let next = state.peek_next_n(1);
                if next == Some('+') || next == Some('-') || next.map(|d| d.is_ascii_digit()).unwrap_or(false) {
                    is_float = true;
                    state.advance(1);
                    if let Some(sign) = state.peek() {
                        if sign == '+' || sign == '-' {
                            state.advance(1);
                        }
                    }
                    while let Some(d) = state.peek() {
                        if d.is_ascii_digit() || d == '_' {
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
        state.add_token(if is_float { SqlTokenType::FloatLiteral } else { SqlTokenType::NumberLiteral }, start, end);
        true
    }

    fn lex_identifier_or_keyword<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let ch = match state.current() {
            Some(c) => c,
            None => return false,
        };

        if !ch.is_alphabetic() && ch != '_' {
            return false;
        }

        state.advance(ch.len_utf8());
        while let Some(c) = state.peek() {
            if c.is_alphanumeric() || c == '_' {
                state.advance(c.len_utf8());
            }
            else {
                break;
            }
        }

        let end = state.get_position();
        let text = state.source().get_text_in(oak_core::Range { start, end }).to_uppercase();
        let kind = match text.as_str() {
            "SELECT" => SqlTokenType::Select,
            "FROM" => SqlTokenType::From,
            "WHERE" => SqlTokenType::Where,
            "INSERT" => SqlTokenType::Insert,
            "UPDATE" => SqlTokenType::Update,
            "DELETE" => SqlTokenType::Delete,
            "CREATE" => SqlTokenType::Create,
            "DROP" => SqlTokenType::Drop,
            "ALTER" => SqlTokenType::Alter,
            "ADD" => SqlTokenType::Add,
            "COLUMN" => SqlTokenType::Column,
            "TABLE" => SqlTokenType::Table,
            "VIEW" => SqlTokenType::View,
            "INDEX" => SqlTokenType::Index,
            "INTO" => SqlTokenType::Into,
            "VALUES" => SqlTokenType::Values,
            "SET" => SqlTokenType::Set,
            "JOIN" => SqlTokenType::Join,
            "INNER" => SqlTokenType::Inner,
            "LEFT" => SqlTokenType::Left,
            "RIGHT" => SqlTokenType::Right,
            "FULL" => SqlTokenType::Full,
            "OUTER" => SqlTokenType::Outer,
            "ON" => SqlTokenType::On,
            "AND" => SqlTokenType::And,
            "OR" => SqlTokenType::Or,
            "NOT" => SqlTokenType::Not,
            "NULL" => SqlTokenType::Null,
            "TRUE" => SqlTokenType::True,
            "FALSE" => SqlTokenType::False,
            "TRIGGER" => SqlTokenType::Trigger,
            "AFTER" => SqlTokenType::After,
            "DELIMITER" => SqlTokenType::Delimiter,
            "FOR" => SqlTokenType::For,
            "EACH" => SqlTokenType::Each,
            "ROW" => SqlTokenType::Row,
            "CHECK" => SqlTokenType::Check,
            "BEGIN" => SqlTokenType::Begin,
            "COMMIT" => SqlTokenType::Commit,
            "ROLLBACK" => SqlTokenType::Rollback,
            "TRANSACTION" => SqlTokenType::Transaction,
            "END" => SqlTokenType::End,
            "IF" => SqlTokenType::If,
            "EXISTS" => SqlTokenType::Exists,
            "RENAME" => SqlTokenType::Rename,
            "TO" => SqlTokenType::To,
            "AS" => SqlTokenType::As,
            "BY" => SqlTokenType::By,
            "ORDER" => SqlTokenType::Order,
            "ASC" => SqlTokenType::Asc,
            "DESC" => SqlTokenType::Desc,
            "GROUP" => SqlTokenType::Group,
            "HAVING" => SqlTokenType::Having,
            "LIMIT" => SqlTokenType::Limit,
            "OFFSET" => SqlTokenType::Offset,
            "UNION" => SqlTokenType::Union,
            "ALL" => SqlTokenType::All,
            "DISTINCT" => SqlTokenType::Distinct,
            "PRIMARY" => SqlTokenType::Primary,
            "KEY" => SqlTokenType::Key,
            "FOREIGN" => SqlTokenType::Foreign,
            "REFERENCES" => SqlTokenType::References,
            "DEFAULT" => SqlTokenType::Default,
            "UNIQUE" => SqlTokenType::Unique,
            "AUTO_INCREMENT" | "AUTOINCREMENT" => SqlTokenType::AutoIncrement,
            "INT" => SqlTokenType::Int,
            "INTEGER" => SqlTokenType::Integer,
            "VARCHAR" => SqlTokenType::Varchar,
            "CHAR" => SqlTokenType::Char,
            "TEXT" => SqlTokenType::Text,
            "DATE" => SqlTokenType::Date,
            "TIME" => SqlTokenType::Time,
            "TIMESTAMP" => SqlTokenType::Timestamp,
            "DECIMAL" => SqlTokenType::Decimal,
            "FLOAT" => SqlTokenType::Float,
            "DOUBLE" => SqlTokenType::Double,
            "BOOLEAN" => SqlTokenType::Boolean,
            "SERIAL" => SqlTokenType::Serial,
            "BIGSERIAL" => SqlTokenType::BigSerial,
            "CONFLICT" => SqlTokenType::Conflict,
            "DO" => SqlTokenType::Do,
            "NOTHING" => SqlTokenType::Nothing,
            "RETURNING" => SqlTokenType::Returning,
            "ILIKE" => SqlTokenType::Ilike,
            "STRICT" => SqlTokenType::Strict,
            "WITHOUT" => SqlTokenType::Without,
            "ROWID" => SqlTokenType::Rowid,
            "MAX" => SqlTokenType::Max,
            "EXPLAIN" => SqlTokenType::Explain,
            "PRAGMA" => SqlTokenType::Pragma,
            "SHOW" => SqlTokenType::Show,
            "DATABASE" => SqlTokenType::Database,
            "SCHEMA" => SqlTokenType::Schema,
            "VECTOR" => SqlTokenType::Vector,
            _ => SqlTokenType::Identifier_,
        };

        state.add_token(kind, start, end);
        true
    }

    fn lex_operators<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();

        let ops = [
            ("::", SqlTokenType::DoubleColon),
            ("||", SqlTokenType::Concat),
            ("<=", SqlTokenType::LessEqual),
            (">=", SqlTokenType::GreaterEqual),
            ("<>", SqlTokenType::NotEqual),
            ("!=", SqlTokenType::NotEqual),
            ("=", SqlTokenType::Equal),
            ("<", SqlTokenType::Less),
            (">", SqlTokenType::Greater),
            ("+", SqlTokenType::Plus),
            ("-", SqlTokenType::Minus),
            ("*", SqlTokenType::Star),
            ("/", SqlTokenType::Slash),
            ("%", SqlTokenType::Percent),
        ];

        for (op, kind) in ops {
            if state.starts_with(op) {
                state.advance(op.len());
                state.add_token(kind, start, state.get_position());
                return true;
            }
        }

        false
    }

    fn lex_single_char_tokens<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start = state.get_position();
        let ch = match state.current() {
            Some(c) => c,
            None => return false,
        };

        let kind = match ch {
            '(' => SqlTokenType::LeftParen,
            ')' => SqlTokenType::RightParen,
            ',' => SqlTokenType::Comma,
            ';' => SqlTokenType::Semicolon,
            '.' => SqlTokenType::Dot,
            ':' => SqlTokenType::Colon,
            '[' => SqlTokenType::LeftBracket,
            ']' => SqlTokenType::RightBracket,
            _ => return false,
        };

        state.advance(ch.len_utf8());
        state.add_token(kind, start, state.get_position());
        true
    }
}
