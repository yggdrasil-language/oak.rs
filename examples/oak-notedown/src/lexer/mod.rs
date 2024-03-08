#![doc = include_str!("readme.md")]
/// Token type definitions for the Notedown lexer.
pub mod token_type;

use crate::{language::NotedownLanguage, lexer::token_type::NoteTokenType};
use oak_core::{Lexer, LexerCache, LexerState, lexer::LexOutput, source::Source};

pub(crate) type State<'a, S> = LexerState<'a, S, NotedownLanguage>;

/// Notedown lexer implementation
#[derive(Clone, Debug)]
pub struct NotedownLexer<'config> {
    config: &'config NotedownLanguage,
}

impl<'config> NotedownLexer<'config> {
    /// Create a new lexer with the given language configuration
    pub fn new(config: &'config NotedownLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Lexer<NotedownLanguage> for NotedownLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &'a S, _edits: &[oak_core::TextEdit], cache: &'a mut impl LexerCache<NotedownLanguage>) -> LexOutput<NotedownLanguage> {
        let mut state = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> NotedownLexer<'config> {
    fn run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> Result<(), oak_core::OakError> {
        while state.not_at_end() {
            // Try various lexical rules
            if self.skip_whitespace(state) {
                continue;
            }

            if self.lex_newline(state) {
                continue;
            }

            if self.lex_heading(state) {
                continue;
            }

            if self.lex_code_block(state) {
                continue;
            }

            if self.lex_inline_code(state) {
                continue;
            }

            if self.lex_strikethrough(state) {
                continue;
            }

            if self.lex_emphasis(state) {
                continue;
            }

            if self.lex_link_or_image(state) {
                continue;
            }

            if self.lex_task_marker(state) {
                continue;
            }

            if self.lex_list_marker(state) {
                continue;
            }

            if self.lex_blockquote(state) {
                continue;
            }

            if self.lex_horizontal_rule(state) {
                continue;
            }

            if self.lex_special_char(state) {
                continue;
            }

            if self.lex_text(state) {
                continue;
            }

            // If no rules match, skip the current character
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
            }
        }
        Ok(())
    }

    /// Skips whitespace
    fn skip_whitespace<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
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
            state.add_token(NoteTokenType::Whitespace, start_pos, state.get_position());
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
            state.add_token(NoteTokenType::Newline, start_pos, state.get_position());
            true
        }
        else if let Some('\r') = state.peek() {
            state.advance(1);
            if let Some('\n') = state.peek() {
                state.advance(1);
            }
            state.add_token(NoteTokenType::Newline, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles headings
    fn lex_heading<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if at the beginning of a line
        if start_pos > 0 {
            if let Some(prev_char) = state.get_char_at(start_pos - 1) {
                if prev_char != '\n' && prev_char != '\r' {
                    return false;
                }
            }
        }

        if let Some('#') = state.peek() {
            let mut level = 0;
            let mut pos = start_pos;

            // Count the number of #
            while let Some('#') = state.get_char_at(pos) {
                level += 1;
                pos += 1;
                if level > 6 {
                    return false; // More than 6 levels of heading is not a valid heading
                }
            }

            // Check if there is a space after #
            if let Some(ch) = state.get_char_at(pos) {
                if ch != ' ' && ch != '\t' && ch != '\n' && ch != '\r' {
                    return false;
                }
            }

            state.advance(level);

            let heading_kind = match level {
                1 => NoteTokenType::Heading1,
                2 => NoteTokenType::Heading2,
                3 => NoteTokenType::Heading3,
                4 => NoteTokenType::Heading4,
                5 => NoteTokenType::Heading5,
                6 => NoteTokenType::Heading6,
                _ => return false,
            };

            state.add_token(heading_kind, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles inline code
    fn lex_inline_code<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('`') = state.peek() {
            state.advance(1);
            let mut found_end = false;

            while let Some(ch) = state.peek() {
                if ch == '`' {
                    state.advance(1);
                    found_end = true;
                    break;
                }
                else if ch == '\n' || ch == '\r' {
                    break; // Inline code cannot span multiple lines
                }
                else {
                    state.advance(ch.len_utf8());
                }
            }

            if found_end {
                state.add_token(NoteTokenType::InlineCode, start_pos, state.get_position());
                true
            }
            else {
                // Backtrack to start position
                state.set_position(start_pos);
                false
            }
        }
        else {
            false
        }
    }

    /// Handles code blocks
    fn lex_code_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if at the beginning of a line
        if start_pos > 0 {
            if let Some(prev_char) = state.get_char_at(start_pos - 1) {
                if prev_char != '\n' && prev_char != '\r' {
                    return false;
                }
            }
        }

        // Check if it's a fence like ``` or ~~~
        let fence_char = if let Some('`') = state.peek() {
            '`'
        }
        else if let Some('~') = state.peek() {
            '~'
        }
        else {
            return false;
        };

        let mut fence_count = 0;
        let mut pos = start_pos;

        // Count fence characters
        while let Some(ch) = state.get_char_at(pos) {
            if ch == fence_char {
                fence_count += 1;
                pos += 1;
            }
            else {
                break;
            }
        }

        if fence_count < 3 {
            return false; // At least 3 fence characters are required
        }

        state.advance(fence_count);
        state.add_token(NoteTokenType::CodeFence, start_pos, state.get_position());

        // Handle language identifier
        let lang_start = state.get_position();
        while let Some(ch) = state.peek() {
            if ch == '\n' || ch == '\r' {
                break;
            }
            else if ch != ' ' && ch != '\t' {
                state.advance(ch.len_utf8());
            }
            else {
                break;
            }
        }

        if state.get_position() > lang_start {
            state.add_token(NoteTokenType::CodeLanguage, lang_start, state.get_position());
        }

        true
    }

    /// Handles bold and italic (emphasis)
    ///
    /// 每个定界符字符单独成 token，连串（如 `**`、`***`）即为多个
    /// `Emphasis` token。这样解析器才能按需把连串拆给相邻的强调元素，
    /// 不会因为一个不可分割的 `Strong` token 同时充当内外两层的收尾。
    fn lex_emphasis<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if !matches!(state.peek(), Some('*') | Some('_')) {
            return false;
        }

        state.advance(1);
        state.add_token(NoteTokenType::Emphasis, start_pos, state.get_position());
        true
    }

    /// Handles strikethrough
    fn lex_strikethrough<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('~') = state.peek() {
            if let Some('~') = state.get_char_at(start_pos + 1) {
                state.advance(2);
                state.add_token(NoteTokenType::Strikethrough, start_pos, state.get_position());
                true
            }
            else {
                false
            }
        }
        else {
            false
        }
    }

    /// Handles links and images
    fn lex_link_or_image<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if it's an image ![
        let is_image = if let Some('!') = state.peek() {
            state.advance(1);
            true
        }
        else {
            false
        };

        if let Some('[') = state.peek() {
            state.advance(1);

            let token_kind = if is_image { NoteTokenType::Image } else { NoteTokenType::Link };

            state.add_token(token_kind, start_pos, state.get_position());
            true
        }
        else {
            if is_image {
                // Backtrack exclamation mark
                state.set_position(start_pos);
            }
            false
        }
    }

    /// Handles list markers
    fn lex_list_marker<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if at the beginning of a line or only whitespace before
        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.get_char_at(check_pos) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                else if ch != ' ' && ch != '\t' {
                    return false; // Non-whitespace characters before
                }
            }
        }

        if let Some(ch) = state.peek() {
            match ch {
                '-' | '*' | '+' => {
                    // Unordered list
                    state.advance(1);
                    if let Some(next_ch) = state.peek() {
                        if next_ch == ' ' || next_ch == '\t' {
                            state.add_token(NoteTokenType::ListMarker, start_pos, state.get_position());
                            return true;
                        }
                    }
                    state.set_position(start_pos);
                    false
                }
                '0'..='9' => {
                    // Ordered list
                    while let Some(digit) = state.peek() {
                        if digit.is_ascii_digit() {
                            state.advance(1);
                        }
                        else {
                            break;
                        }
                    }

                    if let Some('.') = state.peek() {
                        state.advance(1);
                        if let Some(next_ch) = state.peek() {
                            if next_ch == ' ' || next_ch == '\t' {
                                state.add_token(NoteTokenType::ListMarker, start_pos, state.get_position());
                                return true;
                            }
                        }
                    }

                    state.set_position(start_pos);
                    false
                }
                _ => false,
            }
        }
        else {
            false
        }
    }

    /// Handles task list markers
    fn lex_task_marker<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some('[') = state.peek() {
            state.advance(1);
            if let Some(ch) = state.peek() {
                if ch == ' ' || ch == 'x' || ch == 'X' {
                    state.advance(1);
                    if let Some(']') = state.peek() {
                        state.advance(1);
                        state.add_token(NoteTokenType::TaskMarker, start_pos, state.get_position());
                        return true;
                    }
                }
            }
            state.set_position(start_pos);
        }
        false
    }

    /// Handles blockquotes
    fn lex_blockquote<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if at the beginning of a line or only whitespace before
        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.get_char_at(check_pos) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                else if ch != ' ' && ch != '\t' {
                    return false;
                }
            }
        }

        if let Some('>') = state.peek() {
            state.advance(1);
            state.add_token(NoteTokenType::BlockquoteMarker, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles horizontal rules
    fn lex_horizontal_rule<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        // Check if at the beginning of a line or only whitespace before
        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.get_char_at(check_pos) {
                if ch == '\n' || ch == '\r' {
                    break;
                }
                else if ch != ' ' && ch != '\t' {
                    return false;
                }
            }
        }

        if let Some(ch) = state.peek() {
            if ch == '-' || ch == '*' || ch == '_' {
                let rule_char = ch;
                let mut count = 0;
                let mut pos = start_pos;

                // Count the number of consecutive rule characters
                while let Some(current_ch) = state.get_char_at(pos) {
                    if current_ch == rule_char {
                        count += 1;
                        pos += 1;
                    }
                    else if current_ch == ' ' || current_ch == '\t' {
                        pos += 1; // Allow spaces
                    }
                    else {
                        break;
                    }
                }

                if count >= 3 {
                    // Check until the end of line
                    while let Some(current_ch) = state.get_char_at(pos) {
                        if current_ch == '\n' || current_ch == '\r' {
                            break;
                        }
                        else if current_ch == ' ' || current_ch == '\t' {
                            pos += 1;
                        }
                        else {
                            return false; // Other characters at the end of line
                        }
                    }

                    state.set_position(pos);
                    state.add_token(NoteTokenType::HorizontalRule, start_pos, state.get_position());
                    return true;
                }
            }
        }
        false
    }

    /// Handles special characters
    fn lex_special_char<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let token_kind = match ch {
                '[' => NoteTokenType::LeftBracket,
                ']' => NoteTokenType::RightBracket,
                '(' => NoteTokenType::LeftParen,
                ')' => NoteTokenType::RightParen,
                '<' => NoteTokenType::LeftAngle,
                '>' => NoteTokenType::RightAngle,
                '*' => NoteTokenType::Asterisk,
                '_' => NoteTokenType::Underscore,
                '`' => NoteTokenType::Backtick,
                '~' => NoteTokenType::Tilde,
                '#' => NoteTokenType::Hash,
                '|' => NoteTokenType::Pipe,
                '-' => NoteTokenType::Dash,
                '+' => NoteTokenType::Plus,
                '.' => NoteTokenType::Dot,
                ':' => NoteTokenType::Colon,
                '!' => NoteTokenType::Exclamation,
                '\\' => NoteTokenType::Escape,
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

    /// Handles normal text
    fn lex_text<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) -> bool {
        let start_pos = state.get_position();

        while let Some(ch) = state.peek() {
            // Stop when special characters are encountered
            match ch {
                ' ' | '\t' | '\n' | '\r' | '#' | '*' | '_' | '`' | '~' | '[' | ']' | '(' | ')' | '<' | '>' | '|' | '-' | '+' | '.' | ':' | '!' | '\\' => break,
                _ => state.advance(ch.len_utf8()),
            }
        }

        if state.get_position() > start_pos {
            state.add_token(NoteTokenType::Text, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }
}
