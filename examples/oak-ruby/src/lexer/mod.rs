#![doc = include_str!("readme.md")]
/// Token types for the Ruby language.
pub mod token_type;

use crate::{language::RubyLanguage, lexer::token_type::RubyTokenType};
use oak_core::{LexOutput, Lexer, LexerCache, LexerState, OakError, Source, TextEdit};

pub(crate) type State<'a, S> = LexerState<'a, S, RubyLanguage>;

/// A lexer for the Ruby language.
#[derive(Clone, Debug)]
pub struct RubyLexer<'config> {
    config: &'config RubyLanguage,
}

impl<'config> Lexer<RubyLanguage> for RubyLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<RubyLanguage>) -> LexOutput<RubyLanguage> {
        let mut state: State<'_, S> = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof()
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> RubyLexer<'config> {
    /// Creates a new `RubyLexer` with the given configuration.
    pub fn new(config: &'config RubyLanguage) -> Self {
        Self { config }
    }

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

            if self.lex_symbol(state) {
                continue;
            }

            if self.lex_sigil_variable(state) {
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

            state.advance_if_dead_lock(safe_point)
        }

        Ok(())
    }

    /// Skips whitespace characters
    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        while let Some(ch) = state.peek() {
            if ch == ' ' || ch == '\t' { state.advance(ch.len_utf8()) } else { break }
        }

        if state.get_position() > start_pos {
            state.add_token(RubyTokenType::Whitespace, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles newlines
    fn lex_newline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('\n') = state.peek() {
            state.advance(1);
            state.add_token(RubyTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1)
            }
            state.add_token(RubyTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles comments
    fn skip_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        if let Some('#') = state.peek() {
            let start_pos = state.get_position();
            state.advance(1); // Skip '#'

            // Read to end of line
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8())
            }

            state.add_token(RubyTokenType::Comment, start_pos, state.get_position());
            true
        } else if self.skip_begin_end_comment(state) {
            true
        } else {
            false
        }
    }

    /// `=begin` … `=end` 块注释。
    fn skip_begin_end_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();
        if state.peek() != Some('=') {
            return false;
        }
        // 仅在行首或空白后识别；LexerState 无列信息时，要求前面已是换行/文件头。
        let rest = {
            let mut buf = String::new();
            let mut i = 0;
            while let Some(ch) = state.peek_next_n(i) {
                if ch == '\n' || ch == '\r' || buf.len() > 8 {
                    break;
                }
                buf.push(ch);
                i += 1;
            }
            buf
        };
        if !rest.starts_with("=begin") {
            return false;
        }
        // 吃掉到 =end 行结束
        while state.not_at_end() {
            // 找行首 =end
            let line_start = state.get_position();
            let mut line = String::new();
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                line.push(ch);
                state.advance(ch.len_utf8());
            }
            // 吃换行
            if let Some('\r') = state.peek() {
                state.advance(1);
            }
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            if line.trim_start().starts_with("=end") {
                state.add_token(RubyTokenType::Comment, start_pos, state.get_position());
                let _ = line_start;
                return true;
            }
            if state.get_position() == line_start {
                // 死锁保护
                state.advance(1);
            }
        }
        state.add_token(RubyTokenType::Comment, start_pos, state.get_position());
        true
    }

    /// Handles string literals
    fn lex_string_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if it's the start of a string
        let quote_char = match state.peek() {
            Some('"') => '"',
            Some('\'') => '\'',
            Some('`') => '`',
            _ => return false,
        };

        state.advance(1); // Skip the starting quote
        let mut escaped = false;
        while let Some(ch) = state.peek() {
            if escaped {
                escaped = false;
                state.advance(ch.len_utf8());
                continue;
            }

            if ch == '\\' {
                escaped = true;
                state.advance(1);
                continue;
            }

            if ch == quote_char {
                state.advance(1); // Skip the ending quote
                break;
            }
            else if ch == '\n' || ch == '\r' {
                // Ruby strings can span multiple lines
                state.advance(ch.len_utf8())
            }
            else {
                state.advance(ch.len_utf8())
            }
        }

        state.add_token(RubyTokenType::StringLiteral, start_pos, state.get_position());
        true
    }

    /// Handles symbols
    fn lex_symbol<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        if let Some(':') = state.peek() {
            // `::` 是作用域运算符，必须留给 `lex_single_char_tokens`。
            if state.peek_next_n(1) == Some(':') {
                return false;
            }
            let start_pos = state.get_position();
            state.advance(1); // Skip ':'

            // Check if the next character is the start of an identifier
            if let Some(ch) = state.peek() {
                if ch.is_ascii_alphabetic() || ch == '_' {
                    // Read identifier
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '?' || ch == '!' { state.advance(1) } else { break }
                    }
                    state.add_token(RubyTokenType::Symbol, start_pos, state.get_position());
                    return true;
                }
                else if ch == '"' || ch == '\'' {
                    // Quoted symbol
                    let quote = ch;
                    state.advance(1);

                    let mut escaped = false;
                    while let Some(ch) = state.peek() {
                        if escaped {
                            escaped = false;
                            state.advance(ch.len_utf8());
                            continue;
                        }

                        if ch == '\\' {
                            escaped = true;
                            state.advance(1);
                            continue;
                        }

                        if ch == quote {
                            state.advance(1);
                            break;
                        }
                        else {
                            state.advance(ch.len_utf8())
                        }
                    }
                    state.add_token(RubyTokenType::Symbol, start_pos, state.get_position());
                    return true;
                }
                // 单独 `:` 不是 Symbol：回退，交给单字符 tokenize。
                state.set_position(start_pos);
            }
        }
        false
    }

    /// Handles number literals
    fn lex_number_literal<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if !state.peek().map_or(false, |c| c.is_ascii_digit()) {
            return false;
        }

        let mut is_float = false;

        // Check for base prefix
        if state.peek() == Some('0') {
            let next_char = state.peek_next_n(1);
            match next_char {
                Some('b') | Some('B') => {
                    state.advance(2); // Skip '0b' or '0B'
                    // Read binary number
                    while let Some(ch) = state.peek() {
                        if ch == '0' || ch == '1' {
                            state.advance(1);
                        }
                        else if ch == '_' {
                            state.advance(1); // Digit separator
                        }
                        else {
                            break;
                        }
                    }
                }
                Some('o') | Some('O') => {
                    state.advance(2); // Skip '0o' or '0O'
                    // Read octal number
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_digit() && ch < '8' {
                            state.advance(1);
                        }
                        else if ch == '_' {
                            state.advance(1); // Digit separator
                        }
                        else {
                            break;
                        }
                    }
                }
                Some('x') | Some('X') => {
                    state.advance(2); // Skip '0x' or '0X'
                    // Read hexadecimal number
                    while let Some(ch) = state.peek() {
                        if ch.is_ascii_hexdigit() {
                            state.advance(1);
                        }
                        else if ch == '_' {
                            state.advance(1); // Digit separator
                        }
                        else {
                            break;
                        }
                    }
                }
                _ => {
                    // Decimal number
                    self.lex_decimal_number(state, &mut is_float)
                }
            }
        }
        else {
            // Decimal number
            self.lex_decimal_number(state, &mut is_float)
        }

        let kind = if is_float { RubyTokenType::FloatLiteral } else { RubyTokenType::IntegerLiteral };

        state.add_token(kind, start_pos, state.get_position());
        true
    }

    /// Handles decimal numbers
    fn lex_decimal_number<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, is_float: &mut bool) {
        // Read integer part
        while let Some(ch) = state.peek() {
            if ch.is_ascii_digit() {
                state.advance(1);
            }
            else if ch == '_' {
                state.advance(1); // Digit separator
            }
            else {
                break;
            }
        }

        // Check for decimal point
        if state.peek() == Some('.') && state.peek_next_n(1).map_or(false, |c| c.is_ascii_digit()) {
            *is_float = true;
            state.advance(1); // Skip the decimal point
            // Read fractional part
            while let Some(ch) = state.peek() {
                if ch.is_ascii_digit() {
                    state.advance(1);
                }
                else if ch == '_' {
                    state.advance(1); // Digit separator
                }
                else {
                    break;
                }
            }
        }

        // Check for scientific notation
        if let Some('e') | Some('E') = state.peek() {
            *is_float = true;
            state.advance(1);

            // Optional sign
            if let Some('+') | Some('-') = state.peek() {
                state.advance(1);
            }

            // Exponent part
            while let Some(ch) = state.peek() {
                if ch.is_ascii_digit() {
                    state.advance(1);
                }
                else if ch == '_' {
                    state.advance(1); // Digit separator
                }
                else {
                    break;
                }
            }
        }
    }

    /// `@ivar` / `@@cvar` / `$global` 整词。
    fn lex_sigil_variable<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let Some(sigil) = state.peek() else {
            return false;
        };
        if sigil != '@' && sigil != '$' {
            return false;
        }
        let start_pos = state.get_position();
        state.advance(1);
        let mut kind = if sigil == '$' {
            RubyTokenType::GlobalVariable
        } else {
            RubyTokenType::InstanceVariable
        };
        if sigil == '@' && state.peek() == Some('@') {
            state.advance(1);
            kind = RubyTokenType::ClassVariable;
        }
        // `$1` 等 digit 全局；其余需标识符首字符。
        let Some(first) = state.peek() else {
            state.set_position(start_pos);
            return false;
        };
        if !(first.is_ascii_alphabetic() || first == '_' || (sigil == '$' && first.is_ascii_digit())) {
            state.set_position(start_pos);
            return false;
        }
        state.advance(1);
        while let Some(ch) = state.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '?' || ch == '!' {
                state.advance(1);
            } else {
                break;
            }
        }
        state.add_token(kind, start_pos, state.get_position());
        true
    }

    /// Handles identifiers or keywords
    fn lex_identifier_or_keyword<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check the first character
        if !state.peek().map_or(false, |c| c.is_ascii_alphabetic() || c == '_') {
            return false;
        }

        // Build identifier string
        let mut buf = String::new();

        // Read identifier
        while let Some(ch) = state.peek() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '?' || ch == '!' {
                buf.push(ch);
                state.advance(1);
            }
            else {
                break;
            }
        }

        // Check if it's a keyword
        let kind = match buf.as_str() {
            "if" => RubyTokenType::If,
            "unless" => RubyTokenType::Unless,
            "elsif" => RubyTokenType::Elsif,
            "else" => RubyTokenType::Else,
            "case" => RubyTokenType::Case,
            "when" => RubyTokenType::When,
            "then" => RubyTokenType::Then,
            "for" => RubyTokenType::For,
            "while" => RubyTokenType::While,
            "until" => RubyTokenType::Until,
            "break" => RubyTokenType::Break,
            "next" => RubyTokenType::Next,
            "redo" => RubyTokenType::Redo,
            "retry" => RubyTokenType::Retry,
            "return" => RubyTokenType::Return,
            "yield" => RubyTokenType::Yield,
            "def" => RubyTokenType::Def,
            "class" => RubyTokenType::Class,
            "module" => RubyTokenType::Module,
            "end" => RubyTokenType::End,
            "lambda" => RubyTokenType::Lambda,
            "proc" => RubyTokenType::Proc,
            "begin" => RubyTokenType::Begin,
            "rescue" => RubyTokenType::Rescue,
            "ensure" => RubyTokenType::Ensure,
            "raise" => RubyTokenType::Raise,
            "require" => RubyTokenType::Require,
            "load" => RubyTokenType::Load,
            "include" => RubyTokenType::Include,
            "extend" => RubyTokenType::Extend,
            "prepend" => RubyTokenType::Prepend,
            "and" => RubyTokenType::And,
            "or" => RubyTokenType::Or,
            "not" => RubyTokenType::Not,
            "in" => RubyTokenType::In,
            "true" => RubyTokenType::True,
            "false" => RubyTokenType::False,
            "nil" => RubyTokenType::Nil,
            "super" => RubyTokenType::Super,
            "self" => RubyTokenType::Self_,
            "alias" => RubyTokenType::Alias,
            "undef" => RubyTokenType::Undef,
            "defined?" => RubyTokenType::Defined,
            "do" => RubyTokenType::Do,
            _ if buf.starts_with(|c: char| c.is_ascii_uppercase()) => RubyTokenType::Constant,
            _ => RubyTokenType::Identifier,
        };

        state.add_token(kind, start_pos, state.get_position());
        true
    }

    /// Handles operators
    fn lex_operators<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Try to match multi-character operators
        let three_char_ops = ["<=>", "===", "**=", "<<=", ">>=", "||=", "&&=", "..."];
        for op in &three_char_ops {
            if state.peek() == op.chars().nth(0) && state.peek_next_n(1) == op.chars().nth(1) && state.peek_next_n(2) == op.chars().nth(2) {
                state.advance(3);
                let kind = match *op {
                    "<=>" => RubyTokenType::Spaceship,
                    "===" => RubyTokenType::EqualEqualEqual,
                    "**=" => RubyTokenType::PowerAssign,
                    "<<=" => RubyTokenType::LeftShiftAssign,
                    ">>=" => RubyTokenType::RightShiftAssign,
                    "||=" => RubyTokenType::OrOrAssign,
                    "&&=" => RubyTokenType::AndAndAssign,
                    "..." => RubyTokenType::DotDotDot,
                    _ => RubyTokenType::Invalid,
                };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        let two_char_ops = ["**", "<<", ">>", "<=", ">=", "==", "!=", "=~", "!~", "&&", "||", "+=", "-=", "*=", "/=", "%=", "&=", "|=", "^=", "..", "=>"];
        for op in &two_char_ops {
            if state.peek() == op.chars().nth(0) && state.peek_next_n(1) == op.chars().nth(1) {
                state.advance(2);
                let kind = match *op {
                    "**" => RubyTokenType::Power,
                    "<<" => RubyTokenType::LeftShift,
                    ">>" => RubyTokenType::RightShift,
                    "<=" => RubyTokenType::LessEqual,
                    ">=" => RubyTokenType::GreaterEqual,
                    "==" => RubyTokenType::EqualEqual,
                    "!=" => RubyTokenType::NotEqual,
                    "=~" => RubyTokenType::Match,
                    "!~" => RubyTokenType::NotMatch,
                    "&&" => RubyTokenType::AndAnd,
                    "||" => RubyTokenType::OrOr,
                    "+=" => RubyTokenType::PlusAssign,
                    "-=" => RubyTokenType::MinusAssign,
                    "*=" => RubyTokenType::MultiplyAssign,
                    "/=" => RubyTokenType::DivideAssign,
                    "%=" => RubyTokenType::ModuloAssign,
                    "&=" => RubyTokenType::AndAssign,
                    "|=" => RubyTokenType::OrAssign,
                    "^=" => RubyTokenType::XorAssign,
                    ".." => RubyTokenType::DotDot,
                    "=>" => RubyTokenType::EqualGreater,
                    _ => RubyTokenType::Invalid,
                };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        // Try to match single-character operators
        let single_char_ops = ['+', '-', '*', '/', '%', '=', '<', '>', '&', '|', '^', '!', '~', '?'];

        if let Some(ch) = state.peek() {
            if single_char_ops.contains(&ch) {
                state.advance(1);
                let kind = match ch {
                    '+' => RubyTokenType::Plus,
                    '-' => RubyTokenType::Minus,
                    '*' => RubyTokenType::Multiply,
                    '/' => RubyTokenType::Divide,
                    '%' => RubyTokenType::Modulo,
                    '=' => RubyTokenType::Assign,
                    '<' => RubyTokenType::Less,
                    '>' => RubyTokenType::Greater,
                    '&' => RubyTokenType::BitAnd,
                    '|' => RubyTokenType::BitOr,
                    '^' => RubyTokenType::Xor,
                    '!' => RubyTokenType::LogicalNot,
                    '~' => RubyTokenType::Tilde,
                    '?' => RubyTokenType::Question,
                    _ => RubyTokenType::Invalid,
                };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        false
    }

    /// Handles delimiters
    fn lex_single_char_tokens<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check for double colon
        if state.peek() == Some(':') && state.peek_next_n(1) == Some(':') {
            state.advance(2);
            state.add_token(RubyTokenType::DoubleColon, start_pos, state.get_position());
            return true;
        }

        // Single-character delimiters（`@`/`$` 由 `lex_sigil_variable` 处理）
        let delimiters = ['(', ')', '[', ']', '{', '}', ',', ';', '.', ':'];

        if let Some(ch) = state.peek() {
            if delimiters.contains(&ch) {
                state.advance(1);
                let kind = match ch {
                    '(' => RubyTokenType::LeftParen,
                    ')' => RubyTokenType::RightParen,
                    '[' => RubyTokenType::LeftBracket,
                    ']' => RubyTokenType::RightBracket,
                    '{' => RubyTokenType::LeftBrace,
                    '}' => RubyTokenType::RightBrace,
                    ',' => RubyTokenType::Comma,
                    ';' => RubyTokenType::Semicolon,
                    '.' => RubyTokenType::Dot,
                    ':' => RubyTokenType::Colon,
                    _ => RubyTokenType::Invalid,
                };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
        }

        // If no known characters are matched, mark as Invalid and advance the position
        if let Some(_ch) = state.peek() {
            state.advance(1);
            state.add_token(RubyTokenType::Invalid, start_pos, state.get_position());
            return true;
        }

        false
    }
}
