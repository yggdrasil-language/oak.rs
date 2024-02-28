#![doc = include_str!("readme.md")]
/// Token types for the Markdown language.
pub mod token_type;

mod block;
mod inline;
mod list;

use crate::{language::MarkdownLanguage, lexer::token_type::MarkdownTokenType};
use oak_core::{Lexer, LexerCache, LexerState, TextEdit, errors::OakError, lexer::LexOutput, source::Source};

pub(crate) type State<'a, S> = LexerState<'a, S, MarkdownLanguage>;

/// Lexer for Markdown language.
#[derive(Clone, Debug)]
pub struct MarkdownLexer<'config> {
    config: &'config MarkdownLanguage,
}

impl<'config> MarkdownLexer<'config> {
    /// Creates a new MarkdownLexer with the given configuration.
    pub fn new(config: &'config MarkdownLanguage) -> Self {
        Self { config }
    }

    fn run<S: Source + ?Sized>(&self, state: &mut State<S>) -> Result<(), OakError> {
        while state.not_at_end() {
            let safe_point = state.get_position();

            if let Some(ch) = state.peek() {
                match ch {
                    ' ' | '\t' => {
                        if self.config.allow_indented_code_blocks && self.lex_indented_code_block(state) {
                            continue;
                        }
                        self.skip_whitespace(state);
                    }
                    '\n' | '\r' => {
                        self.lex_newline(state);
                    }
                    '$' if self.config.allow_math => {
                        if self.lex_math(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '^' if self.config.allow_subscript => {
                        if self.lex_sub_superscript(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '#' => {
                        if self.config.allow_headings && self.lex_heading(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '`' => {
                        if self.config.allow_fenced_code_blocks && self.lex_code_block(state) {
                            continue;
                        }
                        if self.lex_inline_code(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '~' => {
                        if self.lex_code_block(state) {
                            continue;
                        }
                        if self.config.allow_strikethrough && self.lex_strikethrough(state) {
                            continue;
                        }
                        if self.config.allow_subscript && self.lex_sub_superscript(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '*' | '_' => {
                        if self.config.allow_horizontal_rules && self.lex_horizontal_rule(state) {
                            continue;
                        }
                        if self.config.allow_lists && self.lex_list_marker(state) {
                            continue;
                        }
                        if self.lex_emphasis(state) {
                            continue;
                        }
                        if self.config.allow_abbreviations && self.lex_abbreviation(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '-' => {
                        if self.config.allow_front_matter && self.lex_front_matter(state) {
                            continue;
                        }
                        if self.config.allow_horizontal_rules && self.lex_horizontal_rule(state) {
                            continue;
                        }
                        if self.config.allow_lists && self.lex_list_marker(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '+' => {
                        if self.config.allow_lists && self.lex_list_marker(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '!' => {
                        if self.lex_link_or_image(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '[' => {
                        if self.config.allow_footnotes && self.lex_footnote_from_bracket(state) {
                            continue;
                        }
                        if self.config.allow_task_lists && self.lex_task_marker(state) {
                            continue;
                        }
                        if self.lex_link_or_image(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '>' => {
                        if self.config.allow_blockquotes && self.lex_blockquote(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    ':' => {
                        if self.config.allow_definition_lists && self.lex_definition_description(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '|' if self.config.allow_tables => {
                        self.lex_special_char(state);
                    }
                    '0'..='9' => {
                        if self.lex_list_marker(state) {
                            continue;
                        }
                        self.lex_text(state);
                    }
                    '<' => {
                        if self.config.allow_autolinks && self.lex_angle_autolink(state) {
                            continue;
                        }
                        if self.config.allow_mdx && self.lex_mdx_import_export(state) {
                            continue;
                        }
                        if self.config.allow_html && self.lex_html_tag(state) {
                            continue;
                        }
                        if self.config.allow_xml && self.lex_xml_tag(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    '{' => {
                        if self.config.allow_mdx && self.lex_mdx_expression(state) {
                            continue;
                        }
                        self.lex_special_char(state);
                    }
                    ']' | '(' | ')' | '|' | '.' | '\\' => {
                        self.lex_special_char(state);
                    }
                    _ => {
                        if self.lex_text(state) {
                            continue;
                        }
                        let start_pos = state.get_position();
                        state.advance(ch.len_utf8());
                        state.add_token(MarkdownTokenType::Error, start_pos, state.get_position());
                    }
                }
            }

            state.advance_if_dead_lock(safe_point)
        }
        Ok(())
    }

    /// Skips whitespace
    fn skip_whitespace<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
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
            state.add_token(MarkdownTokenType::Whitespace, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles newlines
    fn lex_newline<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if let Some('\n') = state.peek() {
            state.advance(1);
            state.add_token(MarkdownTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            state.add_token(MarkdownTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles HTML tags or comments.
    fn lex_html_tag<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        self.lex_any_tag(state, MarkdownTokenType::HtmlTag, MarkdownTokenType::HtmlComment)
    }

    /// Handles XML tags or comments.
    fn lex_xml_tag<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        self.lex_any_tag(state, MarkdownTokenType::XmlTag, MarkdownTokenType::XmlComment)
    }

    /// Handles MDX import/export statements.
    fn lex_mdx_import_export<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let source = state.source();
        let remaining = source.get_text_in((start_pos..source.length()).into());

        if remaining.starts_with("import ") {
            let mut end_pos = 7;
            let mut in_string = None;

            for (i, ch) in remaining[7..].char_indices() {
                if let Some(quote) = in_string {
                    if ch == quote {
                        in_string = None;
                    }
                }
                else if ch == '"' || ch == '\'' {
                    in_string = Some(ch);
                }
                else if ch == '\n' {
                    end_pos = 7 + i;
                    break;
                }
            }

            if end_pos > 7 {
                state.set_position(start_pos + end_pos);
                state.add_token(MarkdownTokenType::MdxImport, start_pos, start_pos + end_pos);
                return true;
            }
        }

        if remaining.starts_with("export ") {
            let mut end_pos = 7;

            for (i, ch) in remaining[7..].char_indices() {
                if ch == '\n' {
                    end_pos = 7 + i;
                    break;
                }
            }

            state.set_position(start_pos + end_pos);
            state.add_token(MarkdownTokenType::MdxExport, start_pos, start_pos + end_pos);
            return true;
        }

        if remaining.starts_with("<") && !remaining.starts_with("<!--") {
            let mut end_pos = 0;
            let mut depth = 0;
            let mut in_string = None;

            for (i, ch) in remaining.char_indices() {
                if let Some(quote) = in_string {
                    if ch == quote {
                        in_string = None;
                    }
                }
                else if ch == '"' || ch == '\'' {
                    in_string = Some(ch);
                }
                else if ch == '<' {
                    depth += 1;
                }
                else if ch == '>' {
                    depth -= 1;
                    if depth == 0 {
                        end_pos = i + 1;
                        break;
                    }
                }
            }

            if end_pos > 0 {
                state.set_position(start_pos + end_pos);
                state.add_token(MarkdownTokenType::MdxComponent, start_pos, start_pos + end_pos);
                return true;
            }
        }

        false
    }

    /// Handles MDX JSX expressions: `{expression}`.
    fn lex_mdx_expression<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if let Some('{') = state.peek() {
            state.advance(1);
            let mut depth = 1;
            let mut in_string = None;

            while let Some(ch) = state.peek() {
                if let Some(quote) = in_string {
                    if ch == quote {
                        in_string = None;
                    }
                }
                else if ch == '"' || ch == '\'' {
                    in_string = Some(ch);
                }
                else if ch == '{' {
                    depth += 1;
                }
                else if ch == '}' {
                    depth -= 1;
                    if depth == 0 {
                        state.advance(1);
                        state.add_token(MarkdownTokenType::MdxExpression, start_pos, state.get_position());
                        return true;
                    }
                }
                state.advance(ch.len_utf8());
            }
        }

        state.set_position(start_pos);
        false
    }

    /// Common tag handling logic.
    fn lex_any_tag<S: Source + ?Sized>(&self, state: &mut State<S>, tag_kind: MarkdownTokenType, comment_kind: MarkdownTokenType) -> bool {
        let start_pos = state.get_position();

        if let Some('<') = state.peek() {
            state.advance(1);

            if let Some('!') = state.peek() {
                if state.source().get_char_at(state.get_position() + 1) == Some('-') && state.source().get_char_at(state.get_position() + 2) == Some('-') {
                    state.advance(3);
                    let mut found_end = false;
                    while let Some(ch) = state.peek() {
                        if ch == '-' && state.source().get_char_at(state.get_position() + 1) == Some('-') && state.source().get_char_at(state.get_position() + 2) == Some('>') {
                            state.advance(3);
                            found_end = true;
                            break;
                        }
                        state.advance(ch.len_utf8());
                    }
                    if found_end {
                        state.add_token(comment_kind, start_pos, state.get_position());
                        return true;
                    }
                }
            }

            let mut found_end = false;
            let mut in_string = None;

            while let Some(ch) = state.peek() {
                if let Some(quote) = in_string {
                    if ch == quote {
                        in_string = None;
                    }
                }
                else {
                    if ch == '>' {
                        state.advance(1);
                        found_end = true;
                        break;
                    }
                    else if ch == '"' || ch == '\'' {
                        in_string = Some(ch);
                    }
                }
                state.advance(ch.len_utf8());
            }

            if found_end {
                state.add_token(tag_kind, start_pos, state.get_position());
                true
            }
            else {
                state.set_position(start_pos);
                false
            }
        }
        else {
            false
        }
    }

    /// Lexes special characters.
    fn lex_special_char<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let token_kind = match ch {
                '[' => MarkdownTokenType::LBracket,
                ']' => MarkdownTokenType::RBracket,
                '(' => MarkdownTokenType::LParen,
                ')' => MarkdownTokenType::RParen,
                '<' => MarkdownTokenType::Less,
                '>' => MarkdownTokenType::Greater,
                '*' => MarkdownTokenType::Asterisk,
                '_' => MarkdownTokenType::Underscore,
                '`' => MarkdownTokenType::Backtick,
                '~' => MarkdownTokenType::Tilde,
                '#' => MarkdownTokenType::Hash,
                '|' => MarkdownTokenType::Pipe,
                '-' => MarkdownTokenType::Dash,
                '+' => MarkdownTokenType::Plus,
                '.' => MarkdownTokenType::Dot,
                ':' => MarkdownTokenType::Colon,
                '!' => MarkdownTokenType::Exclamation,
                '\\' => MarkdownTokenType::Escape,
                '$' => MarkdownTokenType::Dollar,
                '^' => MarkdownTokenType::Caret,
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

    /// Lexes angle-bracket autolinks such as `<https://example.com>`.
    fn lex_angle_autolink<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();
        if state.peek() != Some('<') {
            return false;
        }

        let mut pos = start_pos + 1;
        while pos < state.source().length() {
            let ch = match state.source().get_char_at(pos) {
                Some(ch) => ch,
                None => break,
            };
            if ch == '>' {
                if pos > start_pos + 1 {
                    state.set_position(pos + 1);
                    state.add_token(MarkdownTokenType::AutoLink, start_pos, pos + 1);
                    return true;
                }
                break;
            }
            if ch == '\n' || ch == '\r' || ch == '<' {
                break;
            }
            pos += 1;
        }

        false
    }

    /// Lexes bare links (HTTP/HTTPS URLs).
    fn lex_auto_link<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        if !self.config.allow_barelinks {
            return false;
        }

        let start_pos = state.get_position();

        if state.source().get_char_at(start_pos) == Some('h') && state.source().get_char_at(start_pos + 1) == Some('t') && state.source().get_char_at(start_pos + 2) == Some('t') && state.source().get_char_at(start_pos + 3) == Some('p') {
            let mut pos = start_pos + 4;
            if state.source().get_char_at(pos) == Some('s') {
                pos += 1;
            }

            if state.source().get_char_at(pos) == Some(':') && state.source().get_char_at(pos + 1) == Some('/') && state.source().get_char_at(pos + 2) == Some('/') {
                pos += 3;

                while pos < state.source().length() {
                    if let Some(ch) = state.source().get_char_at(pos) {
                        if ch.is_alphanumeric() || ch == '-' || ch == '_' || ch == '.' || ch == '/' || ch == '?' || ch == '=' || ch == '&' || ch == '#' || ch == '%' {
                            pos += 1;
                        }
                        else {
                            break;
                        }
                    }
                    else {
                        break;
                    }
                }

                if pos > start_pos + 7 {
                    state.set_position(pos);
                    state.add_token(MarkdownTokenType::AutoLink, start_pos, pos);
                    return true;
                }
            }
        }

        false
    }

    /// Lexes plain text.
    fn lex_text<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        if self.lex_auto_link(state) {
            return true;
        }

        let start_pos = state.get_position();

        while let Some(ch) = state.peek() {
            match ch {
                ' ' | '\t' | '\n' | '\r' | '#' | '*' | '_' | '`' | '~' | '[' | ']' | '(' | ')' | '<' | '>' | '|' | '-' | '+' | '.' | ':' | '!' | '\\' | '$' | '^' => break,
                '{' if self.config.allow_mdx => break,
                _ => {
                    state.advance(ch.len_utf8());
                }
            }
        }

        if state.get_position() > start_pos {
            state.add_token(MarkdownTokenType::Text, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }
}

impl<'config> Lexer<MarkdownLanguage> for MarkdownLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, text: &'a S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<MarkdownLanguage>) -> LexOutput<MarkdownLanguage> {
        let mut state = State::new(text);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> MarkdownLexer<'config> {
    /// Runs the lexer on the given source and returns the output.
    pub fn lex_internal<'a, S: Source + ?Sized>(&self, source: &'a S) -> LexOutput<MarkdownLanguage> {
        let mut state = State::new(source);
        let result = self.run(&mut state);
        state.finish(result)
    }
}
