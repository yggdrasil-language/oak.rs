use crate::lexer::{MarkdownLexer, State, token_type::MarkdownTokenType};
use oak_core::Source;

impl<'config> MarkdownLexer<'config> {
    /// Handles emphasis and strong.
    pub fn lex_emphasis<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let marker_char = if let Some('*') = state.peek() {
            '*'
        }
        else if let Some('_') = state.peek() {
            '_'
        }
        else {
            return false;
        };

        let mut marker_count = 0;
        let mut pos = start_pos;

        while let Some(ch) = state.source().get_char_at(pos) {
            if ch == marker_char {
                marker_count += 1;
                pos += 1;
            }
            else {
                break;
            }
        }

        if marker_count == 0 {
            return false;
        }

        // CommonMark lets `*` open and close inside a word, but `_` cannot:
        // an underscore flanked by alphanumeric characters is both
        // left-flanking and right-flanking, and an underscore opener may not
        // be right-flanking unless it is preceded by punctuation. Treating
        // `snake_case_name` as emphasis would rewrite source text on export, so
        // an intraword `_` run is emitted as literal text instead of a delimiter.
        if marker_char == '_' {
            let before = preceding_char(state.source(), start_pos);
            let after = state.source().get_char_at(pos);
            if before.is_some_and(char::is_alphanumeric) && after.is_some_and(char::is_alphanumeric) {
                state.advance(marker_count);
                state.add_token(MarkdownTokenType::Text, start_pos, state.get_position());
                return true;
            }
        }

        state.advance(marker_count);

        let token_kind = if marker_count >= 2 { MarkdownTokenType::Strong } else { MarkdownTokenType::Emphasis };

        state.add_token(token_kind, start_pos, state.get_position());
        true
    }

    /// Handles inline code.
    pub fn lex_inline_code<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let mut opener_len = 0;
        while let Some('`') = state.source().get_char_at(start_pos + opener_len) {
            opener_len += 1;
        }

        if opener_len == 0 {
            return false;
        }

        state.advance(opener_len);

        while let Some(ch) = state.peek() {
            if ch == '`' {
                // CommonMark: only a closing run of exactly the same length ends the span.
                // Runs of any other length are code span content, so the whole run is
                // consumed instead of being cut into single-backtick delimiters.
                let mut closer_len = 0;
                while let Some('`') = state.source().get_char_at(state.get_position() + closer_len) {
                    closer_len += 1;
                }
                state.advance(closer_len);
                if closer_len == opener_len {
                    state.add_token(MarkdownTokenType::InlineCode, start_pos, state.get_position());
                    return true;
                }
            }
            else if ch == '\n' || ch == '\r' {
                break;
            }
            else {
                state.advance(ch.len_utf8());
            }
        }

        state.set_position(start_pos);
        false
    }

    /// Handles strikethrough.
    pub fn lex_strikethrough<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if let Some('~') = state.peek() {
            if let Some('~') = state.source().get_char_at(start_pos + 1) {
                state.advance(2);
                state.add_token(MarkdownTokenType::Strikethrough, start_pos, state.get_position());
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

    /// Handles links and images.
    pub fn lex_link_or_image<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let is_image = if let Some('!') = state.peek() {
            state.advance(1);
            true
        }
        else {
            false
        };

        if let Some('[') = state.peek() {
            state.advance(1);

            let token_kind = if is_image { MarkdownTokenType::Image } else { MarkdownTokenType::Link };

            state.add_token(token_kind, start_pos, state.get_position());
            true
        }
        else {
            if is_image {
                state.set_position(start_pos);
            }
            false
        }
    }

    /// Lexes math formulas.
    pub fn lex_math<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if let Some('$') = state.peek() {
            state.advance(1);
            let mut is_block = false;

            if let Some('$') = state.peek() {
                state.advance(1);
                is_block = true;
            }

            let mut found_end = false;
            while let Some(ch) = state.peek() {
                if ch == '$' {
                    if is_block {
                        if let Some('$') = state.source().get_char_at(state.get_position() + 1) {
                            state.advance(2);
                            found_end = true;
                            break;
                        }
                    }
                    else {
                        state.advance(1);
                        found_end = true;
                        break;
                    }
                }
                state.advance(ch.len_utf8())
            }

            if found_end {
                let kind = if is_block { MarkdownTokenType::MathBlock } else { MarkdownTokenType::MathInline };
                state.add_token(kind, start_pos, state.get_position());
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

    /// Lexes footnote references and definitions starting at `[`.
    pub fn lex_footnote_from_bracket<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if state.peek() != Some('[') || state.source().get_char_at(start_pos + 1) != Some('^') {
            return false;
        }

        state.advance(1);
        state.advance(1);
        while let Some(ch) = state.peek() {
            if ch == ']' {
                state.advance(1);
                let kind = if state.peek() == Some(':') {
                    state.advance(1);
                    MarkdownTokenType::FootnoteDefinition
                }
                else {
                    MarkdownTokenType::FootnoteReference
                };
                state.add_token(kind, start_pos, state.get_position());
                return true;
            }
            else if ch == '\n' || ch == '\r' {
                break;
            }
            state.advance(ch.len_utf8());
        }

        state.set_position(start_pos);
        false
    }

    /// Lexes superscripts and subscripts.
    pub fn lex_sub_superscript<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if let Some(ch) = state.peek() {
            let marker = ch;
            if marker == '^' || marker == '~' {
                state.advance(1);
                let mut found_end = false;
                while let Some(next_ch) = state.peek() {
                    if next_ch == marker {
                        state.advance(1);
                        found_end = true;
                        break;
                    }
                    else if next_ch == ' ' || next_ch == '\t' || next_ch == '\n' || next_ch == '\r' {
                        break;
                    }
                    state.advance(next_ch.len_utf8())
                }

                if found_end {
                    let kind = if marker == '^' { MarkdownTokenType::Superscript } else { MarkdownTokenType::Subscript };
                    state.add_token(kind, start_pos, state.get_position());
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
        else {
            false
        }
    }
}

/// Character immediately before `offset`, or `None` at the start of the source.
///
/// UTF-8 limits a character to four bytes, so trying the four possible
/// backward widths finds the previous character without scanning the whole
/// prefix.
fn preceding_char<S: Source + ?Sized>(source: &S, offset: usize) -> Option<char> {
    (1..=4).find_map(|back| {
        let start = offset.checked_sub(back)?;
        let text = source.get_text_in((start..offset).into());
        let mut chars = text.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Some(ch),
            _ => None,
        }
    })
}
