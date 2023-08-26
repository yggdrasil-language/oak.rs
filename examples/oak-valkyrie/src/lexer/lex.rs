use crate::{
    ValkyrieLanguage,
    lexer::{ValkyrieKeywords, token_type::ValkyrieTokenType},
};
use oak_core::{
    LexerState, OakError,
    lexer::{CommentConfig, WhitespaceConfig},
    source::Source,
};
use std::sync::LazyLock;
use unicode_ident::{is_xid_continue, is_xid_start};

pub(crate) type State<'a, S> = LexerState<'a, S, ValkyrieLanguage>;

static VK_WHITESPACE: LazyLock<WhitespaceConfig> = LazyLock::new(|| WhitespaceConfig { unicode_whitespace: true });
// Valkyrie 行注释仅为 `#`。不支持 `//` / `///` / `/* */`。
static VK_COMMENT: LazyLock<CommentConfig> =
    LazyLock::new(|| CommentConfig { line_marker: "#", block_start: "", block_end: "", nested_blocks: false });

impl crate::lexer::ValkyrieLexer<'_> {
    /// Runs the lexer on the given state.
    pub(crate) fn run<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let start_pos = state.get_position();

            if self.lex_whitespace(state) || self.lex_comments(state) {
                continue;
            }

            let matched = self.lex_string_literal(state) || self.lex_char_literal(state) || self.lex_number_literal(state) || self.lex_identifier_or_keyword(state) || self.lex_operators(state) || self.lex_single_char_tokens(state);

            if !matched {
                if let Some(c) = state.current() {
                    let char_len = c.len_utf8();
                    state.add_token(ValkyrieTokenType::Error, start_pos, start_pos + char_len);
                    state.advance(char_len);
                }
            }
        }

        Ok(())
    }

    fn lex_whitespace<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let start = state.get_position();
        let range = if VK_WHITESPACE.unicode_whitespace { state.take_while(|c| c.is_whitespace()) } else { state.skip_ascii_whitespace() };

        if range.end > start {
            state.add_token(ValkyrieTokenType::Whitespace, start, range.end);
            true
        }
        else {
            false
        }
    }

    fn lex_comments<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        VK_COMMENT.scan(state, ValkyrieTokenType::LineComment, ValkyrieTokenType::BlockComment)
    }

    fn lex_string_literal<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        let initial_start = state.get_position();

        let prefix_start = state.get_position();
        let mut prefix_end = prefix_start;

        if let Some(ch) = state.current() {
            if is_xid_start(ch) {
                state.advance(ch.len_utf8());
                while let Some(ch) = state.current() {
                    if is_xid_continue(ch) {
                        state.advance(ch.len_utf8());
                    }
                    else {
                        break;
                    }
                }
                prefix_end = state.get_position();
            }
        }

        let has_prefix = prefix_end > prefix_start;

        if has_prefix {
            if let Some('"') = state.current() {
                state.add_token(ValkyrieTokenType::StringPrefix, prefix_start, prefix_end);
                return self.lex_symmetric_string(state, '"', ValkyrieTokenType::StringLiteral);
            }
            else {
                state.set_position(initial_start);
                return false;
            }
        }

        self.lex_symmetric_string(state, '"', ValkyrieTokenType::StringLiteral)
    }

    fn lex_char_literal<S: Source + ?Sized>(&self, state: &mut State<'_, S>) -> bool {
        self.lex_symmetric_string(state, '\'', ValkyrieTokenType::CharLiteral)
    }

    fn lex_symmetric_string<S: Source + ?Sized>(&self, state: &mut State<'_, S>, quote: char, kind: ValkyrieTokenType) -> bool {
        let start = state.get_position();

        let mut quote_count = 0;

        while let Some(c) = state.current() {
            if c == quote {
                quote_count += 1;
                state.advance(c.len_utf8());
            }
            else {
                break;
            }
        }

        if quote_count == 0 {
            state.set_position(start);
            return false;
        }

        if quote_count == 2 {
            state.add_token(kind, start, state.get_position());
            return true;
        }

        let mut current_consecutive = 0;

        while let Some(c) = state.current() {
            if c == quote {
                current_consecutive += 1;
                state.advance(c.len_utf8());
                if current_consecutive == quote_count {
                    let end = state.get_position();
                    state.add_token(kind, start, end);
                    return true;
                }
            }
            else {
                current_consecutive = 0;
                state.advance(c.len_utf8());
            }
        }

        state.add_token(kind, start, state.get_position());
        true
    }

    fn lex_number_literal<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.current() {
            if ch.is_ascii_digit() {
                state.advance(ch.len_utf8());

                while let Some(ch) = state.current() {
                    if ch.is_ascii_digit() || ch == '.' || ch == '_' { state.advance(ch.len_utf8()) } else { break }
                }

                state.add_token(ValkyrieTokenType::IntegerLiteral, start, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_identifier_or_keyword<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.current() {
            if ch == '_' || is_xid_start(ch) {
                state.advance(ch.len_utf8());

                while let Some(ch) = state.current() {
                    if is_xid_continue(ch) { state.advance(ch.len_utf8()) } else { break }
                }

                let end = state.get_position();
                let text = state.get_text_in((start..end).into());
                let token_kind = match &*text {
                    "namespace" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Namespace),
                    "using" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Using),
                    "class" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Class),
                    "abstract" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Abstract),
                    "sealed" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Sealed),
                    "final" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Final),
                    "struct" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Struct),
                    "structure" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Structure),
                    "singleton" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Singleton),
                    "trait" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Trait),
                    "flags" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Flags),
                    "enums" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Enums),
                    "enum" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Enum),
                    "unity" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Unity),
                    "union" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Union),
                    "micro" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Micro),
                    "mezzo" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Mezzo),
                    "macro" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Macro),
                    "widget" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Widget),
                    "let" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Let),
                    "if" => ValkyrieTokenType::Keyword(ValkyrieKeywords::If),
                    "else" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Else),
                    "match" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Match),
                    "case" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Case),
                    "when" => ValkyrieTokenType::Keyword(ValkyrieKeywords::When),
                    "try" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Try),
                    "lambda" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Lambda),
                    "catch" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Catch),
                    "while" => ValkyrieTokenType::Keyword(ValkyrieKeywords::While),
                    "loop" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Loop),
                    "for" => ValkyrieTokenType::Keyword(ValkyrieKeywords::For),
                    "in" => ValkyrieTokenType::Keyword(ValkyrieKeywords::In),
                    "return" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Return),
                    "break" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Break),
                    "continue" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Continue),
                    "true" => ValkyrieTokenType::BoolLiteral,
                    "false" => ValkyrieTokenType::BoolLiteral,
                    "null" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Null),
                    "mut" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Mut),
                    "is" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Is),
                    "type" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Type),
                    "yield" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Yield),
                    "raise" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Raise),
                    "effect" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Effect),
                    "resume" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Resume),
                    "from" => ValkyrieTokenType::Keyword(ValkyrieKeywords::From),
                    "as" => ValkyrieTokenType::Keyword(ValkyrieKeywords::As),
                    "get" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Get),
                    "set" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Set),
                    "Self" => ValkyrieTokenType::Keyword(ValkyrieKeywords::SelfType),
                    "impl" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Impl),
                    "where" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Where),
                    "super" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Super),
                    "override" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Override),
                    "virtual" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Virtual),
                    "readonly" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Readonly),
                    "initiate" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Initiate),
                    "shader" => ValkyrieTokenType::Keyword(ValkyrieKeywords::Shader),
                    "by" => ValkyrieTokenType::Keyword(ValkyrieKeywords::By),
                    "component" if self.config.support_ecs_extension => ValkyrieTokenType::Keyword(ValkyrieKeywords::Component),
                    "system" if self.config.support_ecs_extension => ValkyrieTokenType::Keyword(ValkyrieKeywords::System),
                    "events" if self.config.support_ecs_extension => ValkyrieTokenType::Keyword(ValkyrieKeywords::Events),
                    "_" => ValkyrieTokenType::Underscore,
                    _ => ValkyrieTokenType::Identifier,
                };

                state.add_token(token_kind, start, state.get_position());
                return true;
            }
        }
        false
    }

    fn lex_operators<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.current() {
            let kind = match ch {
                '+' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    ValkyrieTokenType::Plus
                }
                '-' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('>') = state.current() {
                        state.advance('>'.len_utf8());
                        ValkyrieTokenType::Arrow
                    }
                    else {
                        ValkyrieTokenType::Minus
                    }
                }
                '*' => {
                    state.advance(ch.len_utf8());
                    ValkyrieTokenType::Star
                }
                '/' => {
                    state.advance(ch.len_utf8());
                    ValkyrieTokenType::Slash
                }
                '%' => {
                    state.advance(ch.len_utf8());
                    ValkyrieTokenType::Percent
                }
                '=' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('=') = state.current() {
                        state.advance('='.len_utf8());
                        ValkyrieTokenType::EqEq
                    }
                    else {
                        ValkyrieTokenType::Eq
                    }
                }
                '!' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('=') = state.current() {
                        state.advance('='.len_utf8());
                        ValkyrieTokenType::NotEq
                    }
                    else {
                        ValkyrieTokenType::Bang
                    }
                }
                '<' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('=') = state.current() {
                        state.advance('='.len_utf8());
                        ValkyrieTokenType::LessEq
                    }
                    else if let Some('<') = state.current() {
                        state.advance('<'.len_utf8());
                        ValkyrieTokenType::LeftShift
                    }
                    else {
                        ValkyrieTokenType::LessThan
                    }
                }
                '>' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('=') = state.current() {
                        state.advance('='.len_utf8());
                        ValkyrieTokenType::GreaterEq
                    }
                    else if let Some('>') = state.current() {
                        state.advance('>'.len_utf8());
                        ValkyrieTokenType::RightShift
                    }
                    else {
                        ValkyrieTokenType::GreaterThan
                    }
                }
                '&' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('&') = state.current() {
                        state.advance('&'.len_utf8());
                        ValkyrieTokenType::AndAnd
                    }
                    else {
                        ValkyrieTokenType::Ampersand
                    }
                }
                '|' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('|') = state.current() {
                        state.advance('|'.len_utf8());
                        ValkyrieTokenType::OrOr
                    }
                    else {
                        ValkyrieTokenType::Pipe
                    }
                }
                '^' => {
                    state.advance(ch.len_utf8());
                    ValkyrieTokenType::Caret
                }
                '~' => {
                    state.advance(ch.len_utf8());
                    ValkyrieTokenType::Tilde
                }
                '.' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some('.') = state.current() {
                        state.advance('.'.len_utf8());
                        ValkyrieTokenType::DotDot
                    }
                    else {
                        ValkyrieTokenType::Dot
                    }
                }
                ':' => {
                    let ch_len = ch.len_utf8();
                    state.advance(ch_len);
                    if let Some(':') = state.current() {
                        state.advance(':'.len_utf8());
                        ValkyrieTokenType::ColonColon
                    }
                    else if let Some('=') = state.current() {
                        state.advance('='.len_utf8());
                        ValkyrieTokenType::ColonEq
                    }
                    else {
                        ValkyrieTokenType::Colon
                    }
                }
                _ => return false,
            };
            state.add_token(kind, start, state.get_position());
            return true;
        }
        false
    }

    fn lex_single_char_tokens<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let start = state.get_position();
        if let Some(ch) = state.current() {
            match ch {
                '@' => {
                    state.advance(ch.len_utf8());
                    state.add_token(ValkyrieTokenType::At, start, state.get_position());
                    return true;
                }
                _ => {}
            }
            let kind = match ch {
                '(' => ValkyrieTokenType::LeftParen,
                ')' => ValkyrieTokenType::RightParen,
                '{' => ValkyrieTokenType::LeftBrace,
                '}' => ValkyrieTokenType::RightBrace,
                '[' => ValkyrieTokenType::LeftBracket,
                ']' => ValkyrieTokenType::RightBracket,
                ',' => ValkyrieTokenType::Comma,
                ';' => ValkyrieTokenType::Semicolon,
                '$' => ValkyrieTokenType::Dollar,
                '?' => ValkyrieTokenType::Question,
                '⟨' => ValkyrieTokenType::LeftAngle,
                '⟩' => ValkyrieTokenType::RightAngle,
                '⁅' => ValkyrieTokenType::LeftOffset,
                '⁆' => ValkyrieTokenType::RightOffset,
                _ => return false,
            };
            state.advance(ch.len_utf8());
            state.add_token(kind, start, state.get_position());
            return true;
        }
        false
    }
}
