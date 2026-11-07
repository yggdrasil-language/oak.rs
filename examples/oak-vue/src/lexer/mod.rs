#![doc = include_str!("readme.md")]
/// Token types for the Vue language.
pub mod token_type;

use crate::{language::VueLanguage, lexer::token_type::VueTokenType};
use oak_core::{
    Lexer, LexerState,
    lexer::{LexOutput, LexerCache},
    source::Source,
};

/// A lexer for the Vue language.
#[derive(Clone, Debug)]
pub struct VueLexer<'config> {
    config: &'config VueLanguage,
    initial_mode: LexModeKind,
}

pub(crate) type State<'a, S> = LexerState<'a, S, VueLanguage>;

/// The lexical context a Vue single-file component cursor sits in.
///
/// The Vue token stream is flat, so the lexer itself must know whether the
/// cursor sits in plain template text, inside a raw `<script>` / `<style>`
/// body, or inside an expression (`{{ … }}` / a re-lexed directive value).
/// JavaScript comment semantics only exist in the last two; in plain text
/// `/*` is literal content (`serve public/** assets`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LexModeKind {
    /// Template text, tag interiors and SFC root: only `<!-- -->` is a comment.
    Data,
    /// `<script>` / `<style>` content: JavaScript comments are live, so a
    /// `</script>` inside a comment stays inside the block.
    Raw,
    /// Interpolation or directive expression: JavaScript comments are live.
    Expression,
}

/// Mutable lexical context threaded through the token loop.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct LexMode {
    kind: LexModeKind,
    /// Saw `<script` / `<style` and wait for the tag-closing `>` before raw content.
    pending_raw: bool,
}

impl LexMode {
    fn new(kind: LexModeKind) -> Self {
        Self { kind, pending_raw: false }
    }

    /// JavaScript comments (`//`, `/* */`) are only recognized outside plain text.
    fn js_comments(&self) -> bool {
        self.kind != LexModeKind::Data
    }

    /// Tracks context transitions after a token of `kind` was emitted.
    fn advance(&mut self, kind: VueTokenType) {
        use LexModeKind::*;
        match self.kind {
            Data => match kind {
                VueTokenType::ScriptStart | VueTokenType::StyleStart => self.pending_raw = true,
                VueTokenType::Gt if self.pending_raw => {
                    self.kind = Raw;
                    self.pending_raw = false;
                }
                VueTokenType::SlashGt if self.pending_raw => self.pending_raw = false,
                VueTokenType::InterpolationStart => self.kind = Expression,
                _ => {}
            },
            Raw => {
                if kind == VueTokenType::LtSlash {
                    self.kind = Data;
                }
            }
            Expression => {
                if kind == VueTokenType::InterpolationEnd {
                    self.kind = Data;
                }
            }
        }
    }
}

impl<'config> VueLexer<'config> {
    /// Creates a new `VueLexer` for a full single-file component.
    pub fn new(config: &'config VueLanguage) -> Self {
        Self { config, initial_mode: LexModeKind::Data }
    }

    /// Creates a lexer for a standalone expression (directive value, `v-for`).
    ///
    /// The source is one expression, so JavaScript comments stay live from the
    /// first token instead of waiting for an interpolation start marker.
    pub(crate) fn new_expression(config: &'config VueLanguage) -> Self {
        Self { config, initial_mode: LexModeKind::Expression }
    }

    fn lex_token<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, mode: LexMode) -> VueTokenType {
        if self.lex_whitespace(state) {
            return VueTokenType::Whitespace;
        }
        if self.lex_comment(state, mode.js_comments()) {
            return VueTokenType::Comment;
        }

        let start_pos = state.get_position();
        let rest = state.rest();

        if rest.starts_with(&self.config.interpolation_start) {
            state.advance(self.config.interpolation_start.len());
            state.add_token(VueTokenType::InterpolationStart, start_pos, state.get_position());
            return VueTokenType::InterpolationStart;
        }
        if rest.starts_with(&self.config.interpolation_end) {
            state.advance(self.config.interpolation_end.len());
            state.add_token(VueTokenType::InterpolationEnd, start_pos, state.get_position());
            return VueTokenType::InterpolationEnd;
        }

        let ch = match state.peek() {
            Some(c) => c,
            None => {
                state.add_token(VueTokenType::Eof, start_pos, start_pos);
                return VueTokenType::Eof;
            }
        };

        match ch {
            '|' => {
                if state.peek_next_n(1) == Some('|') {
                    state.advance(2);
                    state.add_token(VueTokenType::Or, start_pos, start_pos + 2);
                    VueTokenType::Or
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Pipe, start_pos, start_pos + 1);
                    VueTokenType::Pipe
                }
            }
            '&' => {
                if state.peek_next_n(1) == Some('&') {
                    state.advance(2);
                    state.add_token(VueTokenType::And, start_pos, start_pos + 2);
                    VueTokenType::And
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Amp, start_pos, start_pos + 1);
                    VueTokenType::Amp
                }
            }
            '!' => {
                if state.peek_next_n(1) == Some('=') {
                    state.advance(2);
                    state.add_token(VueTokenType::NotEq, start_pos, start_pos + 2);
                    VueTokenType::NotEq
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Bang, start_pos, start_pos + 1);
                    VueTokenType::Bang
                }
            }
            '+' => {
                state.advance(1);
                state.add_token(VueTokenType::Plus, start_pos, start_pos + 1);
                VueTokenType::Plus
            }
            '-' => {
                state.advance(1);
                state.add_token(VueTokenType::Minus, start_pos, start_pos + 1);
                VueTokenType::Minus
            }
            '*' => {
                state.advance(1);
                state.add_token(VueTokenType::Star, start_pos, start_pos + 1);
                VueTokenType::Star
            }
            '%' => {
                state.advance(1);
                state.add_token(VueTokenType::Percent, start_pos, start_pos + 1);
                VueTokenType::Percent
            }
            '?' => {
                state.advance(1);
                state.add_token(VueTokenType::Question, start_pos, start_pos + 1);
                VueTokenType::Question
            }
            '<' => {
                if state.rest().starts_with("</") {
                    state.advance(2);
                    state.add_token(VueTokenType::LtSlash, start_pos, start_pos + 2);
                    VueTokenType::LtSlash
                }
                else if state.peek_next_n(1) == Some('=') {
                    state.advance(2);
                    state.add_token(VueTokenType::LtEq, start_pos, start_pos + 2);
                    VueTokenType::LtEq
                }
                else if state.rest().starts_with("<!--") {
                    state.advance(4);
                    while state.not_at_end() && !state.rest().starts_with("-->") {
                        state.advance(1);
                    }
                    if state.rest().starts_with("-->") {
                        state.advance(3);
                    }
                    state.add_token(VueTokenType::Comment, start_pos, state.get_position());
                    VueTokenType::Comment
                }
                else if state.rest().starts_with("<!") {
                    state.advance(2);
                    state.add_token(VueTokenType::DocTypeStart, start_pos, start_pos + 2);
                    VueTokenType::DocTypeStart
                }
                else if state.rest().starts_with("<script") && !is_ident_continue(state.peek_next_n(7).unwrap_or(' ')) {
                    state.advance(7);
                    state.add_token(VueTokenType::ScriptStart, start_pos, start_pos + 7);
                    VueTokenType::ScriptStart
                }
                else if state.rest().starts_with("<style") && !is_ident_continue(state.peek_next_n(6).unwrap_or(' ')) {
                    state.advance(6);
                    state.add_token(VueTokenType::StyleStart, start_pos, start_pos + 6);
                    VueTokenType::StyleStart
                }
                else if state.rest().starts_with("<template") && !is_ident_continue(state.peek_next_n(9).unwrap_or(' ')) {
                    state.advance(9);
                    state.add_token(VueTokenType::TemplateStart, start_pos, start_pos + 9);
                    VueTokenType::TemplateStart
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Lt, start_pos, start_pos + 1);
                    VueTokenType::Lt
                }
            }
            '>' => {
                if state.peek_next_n(1) == Some('=') {
                    state.advance(2);
                    state.add_token(VueTokenType::GtEq, start_pos, start_pos + 2);
                    VueTokenType::GtEq
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Gt, start_pos, start_pos + 1);
                    VueTokenType::Gt
                }
            }
            '/' => {
                if state.peek_next_n(1) == Some('>') {
                    state.advance(2);
                    state.add_token(VueTokenType::SlashGt, start_pos, start_pos + 2);
                    VueTokenType::SlashGt
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Slash, start_pos, start_pos + 1);
                    VueTokenType::Slash
                }
            }
            '=' => {
                if state.peek_next_n(1) == Some('>') {
                    state.advance(2);
                    state.add_token(VueTokenType::Arrow, start_pos, start_pos + 2);
                    VueTokenType::Arrow
                }
                else if state.peek_next_n(1) == Some('=') {
                    state.advance(2);
                    state.add_token(VueTokenType::EqEq, start_pos, start_pos + 2);
                    VueTokenType::EqEq
                }
                else {
                    state.advance(1);
                    state.add_token(VueTokenType::Eq, start_pos, start_pos + 1);
                    VueTokenType::Eq
                }
            }
            '{' => {
                state.advance(1);
                state.add_token(VueTokenType::LeftBrace, start_pos, start_pos + 1);
                VueTokenType::LeftBrace
            }
            '}' => {
                state.advance(1);
                state.add_token(VueTokenType::RightBrace, start_pos, start_pos + 1);
                VueTokenType::RightBrace
            }
            '(' => {
                state.advance(1);
                state.add_token(VueTokenType::LeftParen, start_pos, start_pos + 1);
                VueTokenType::LeftParen
            }
            ')' => {
                state.advance(1);
                state.add_token(VueTokenType::RightParen, start_pos, start_pos + 1);
                VueTokenType::RightParen
            }
            '[' => {
                state.advance(1);
                state.add_token(VueTokenType::LeftBracket, start_pos, start_pos + 1);
                VueTokenType::LeftBracket
            }
            ']' => {
                state.advance(1);
                state.add_token(VueTokenType::RightBracket, start_pos, start_pos + 1);
                VueTokenType::RightBracket
            }
            ',' => {
                state.advance(1);
                state.add_token(VueTokenType::Comma, start_pos, start_pos + 1);
                VueTokenType::Comma
            }
            ':' => {
                state.advance(1);
                state.add_token(VueTokenType::Colon, start_pos, start_pos + 1);
                VueTokenType::Colon
            }
            ';' => {
                state.advance(1);
                state.add_token(VueTokenType::Semicolon, start_pos, start_pos + 1);
                VueTokenType::Semicolon
            }
            '.' => {
                state.advance(1);
                state.add_token(VueTokenType::Dot, start_pos, start_pos + 1);
                VueTokenType::Dot
            }
            '@' => {
                state.advance(1);
                state.add_token(VueTokenType::At, start_pos, start_pos + 1);
                VueTokenType::At
            }
            '#' => {
                state.advance(1);
                state.add_token(VueTokenType::Hash, start_pos, start_pos + 1);
                VueTokenType::Hash
            }
            '"' | '\'' => {
                self.lex_string(state);
                VueTokenType::StringLiteral
            }
            _ if ch.is_ascii_digit() => {
                self.lex_number(state);
                VueTokenType::NumberLiteral
            }
            _ if is_ident_start(ch) => self.lex_identifier(state),
            _ => {
                self.lex_text(state);
                VueTokenType::Text
            }
        }
    }

    fn lex_text<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let start_pos = state.get_position();
        while let Some(ch) = state.peek() {
            let rest = state.rest();
            if ch == '<' || rest.starts_with(&self.config.interpolation_start) || ch.is_whitespace() {
                break;
            }
            state.advance(ch.len_utf8())
        }
        if state.get_position() > start_pos {
            state.add_token(VueTokenType::Text, start_pos, state.get_position())
        }
    }

    fn lex_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();
        let mut found = false;
        while let Some(ch) = state.peek() {
            if ch.is_whitespace() {
                state.advance(ch.len_utf8());
                found = true
            }
            else {
                break;
            }
        }
        if found {
            state.add_token(VueTokenType::Whitespace, start_pos, state.get_position())
        }
        found
    }

    fn lex_comment<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, js_comments: bool) -> bool {
        let start_pos = state.get_position();
        if state.rest().starts_with("<!--") {
            state.advance(4);
            while let Some(c) = state.peek() {
                if state.rest().starts_with("-->") {
                    state.advance(3);
                    break;
                }
                state.advance(c.len_utf8())
            }
            state.add_token(VueTokenType::Comment, start_pos, state.get_position());
            return true;
        }
        // JavaScript comments exist in script/style content and in expressions,
        // never in plain template text (`public/**` is literal text there).
        if !js_comments {
            return false;
        }
        if state.rest().starts_with("//") {
            state.advance(2);
            while let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                state.advance(ch.len_utf8())
            }
            state.add_token(VueTokenType::Comment, start_pos, state.get_position());
            return true;
        }
        if state.rest().starts_with("/*") {
            state.advance(2);
            while let Some(ch) = state.peek() {
                if state.rest().starts_with("*/") {
                    state.advance(2);
                    break;
                }
                state.advance(ch.len_utf8())
            }
            state.add_token(VueTokenType::Comment, start_pos, state.get_position());
            return true;
        }
        false
    }

    fn lex_string<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let start_pos = state.get_position();
        let Some(quote) = state.peek()
        else {
            return;
        };
        state.advance(1);
        let mut escaped = false;
        while let Some(ch) = state.peek() {
            if escaped {
                escaped = false;
                state.advance(ch.len_utf8())
            }
            else if ch == '\\' {
                escaped = true;
                state.advance(1)
            }
            else if ch == quote {
                state.advance(1);
                break;
            }
            else {
                state.advance(ch.len_utf8())
            }
        }
        state.add_token(VueTokenType::StringLiteral, start_pos, state.get_position())
    }

    fn lex_number<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let start_pos = state.get_position();
        while let Some(ch) = state.peek() {
            if ch.is_ascii_digit() || ch == '.' { state.advance(1) } else { break }
        }
        state.add_token(VueTokenType::NumberLiteral, start_pos, state.get_position())
    }

    fn lex_identifier<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> VueTokenType {
        let start_pos = state.get_position();
        while let Some(ch) = state.peek() {
            if is_ident_continue(ch) { state.advance(ch.len_utf8()) } else { break }
        }
        let text = state.get_text_in((start_pos..state.get_position()).into());
        let kind = match text.as_ref() {
            "import" => VueTokenType::Import,
            "export" => VueTokenType::Export,
            "default" => VueTokenType::Default,
            "from" => VueTokenType::From,
            "as" => VueTokenType::As,
            "const" => VueTokenType::Const,
            "let" => VueTokenType::Let,
            "var" => VueTokenType::Var,
            "function" => VueTokenType::Function,
            "if" => VueTokenType::If,
            "else" => VueTokenType::Else,
            "while" => VueTokenType::While,
            "for" => VueTokenType::For,
            "return" => VueTokenType::Return,
            "break" => VueTokenType::Break,
            "continue" => VueTokenType::Continue,
            "switch" => VueTokenType::Switch,
            "try" => VueTokenType::Try,
            "throw" => VueTokenType::Throw,
            "in" => VueTokenType::In,
            "of" => VueTokenType::Of,
            "true" => VueTokenType::True,
            "false" => VueTokenType::False,
            "null" => VueTokenType::Null,
            _ => VueTokenType::Identifier,
        };
        state.add_token(kind, start_pos, state.get_position());
        kind
    }
}

fn is_ident_start(ch: char) -> bool {
    ch.is_ascii_alphabetic() || ch == '_' || ch == '$'
}

fn is_ident_continue(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' || ch == '-'
}

impl<'config> Lexer<VueLanguage> for VueLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[oak_core::source::TextEdit], cache: &'a mut impl LexerCache<VueLanguage>) -> LexOutput<VueLanguage> {
        let mut state = State::new_with_cache(source, 0, cache);
        let mut mode = LexMode::new(self.initial_mode);

        while state.not_at_end() {
            let kind = self.lex_token(&mut state, mode);
            mode.advance(kind);
        }

        state.add_eof();

        state.finish_with_cache(Ok(()), cache)
    }
}
