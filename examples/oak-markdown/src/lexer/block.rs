use crate::lexer::{MarkdownLexer, State, token_type::MarkdownTokenType};
use oak_core::Source;

impl<'config> MarkdownLexer<'config> {
    /// Handles headings.
    pub fn lex_heading<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if start_pos > 0 {
            if let Some(prev_char) = state.source().get_char_at(start_pos - 1) {
                if prev_char != '\n' && prev_char != '\r' {
                    return false;
                }
            }
        }

        if let Some('#') = state.peek() {
            let mut level = 0;
            let mut pos = start_pos;

            while let Some('#') = state.source().get_char_at(pos) {
                level += 1;
                pos += 1;
                if level > 6 {
                    return false;
                }
            }

            if let Some(ch) = state.source().get_char_at(pos) {
                if ch != ' ' && ch != '\t' && ch != '\n' && ch != '\r' {
                    return false;
                }
            }

            state.advance(level);

            let heading_kind = match level {
                1 => MarkdownTokenType::Heading1,
                2 => MarkdownTokenType::Heading2,
                3 => MarkdownTokenType::Heading3,
                4 => MarkdownTokenType::Heading4,
                5 => MarkdownTokenType::Heading5,
                6 => MarkdownTokenType::Heading6,
                _ => return false,
            };

            state.add_token(heading_kind, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Handles code blocks.
    pub fn lex_code_block<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        // CommonMark allows a fenced code block to be indented by up to three
        // spaces, so walk back over the leading indentation instead of
        // requiring the previous byte to be a newline.
        let mut line_start = start_pos;
        let mut indent = 0;
        while indent < 4 {
            if line_start == 0 {
                break;
            }
            if state.source().get_char_at(line_start - 1) == Some(' ') {
                line_start -= 1;
                indent += 1;
            }
            else {
                break;
            }
        }

        if indent >= 4 {
            return false;
        }

        if line_start > 0 {
            if let Some(prev_char) = state.source().get_char_at(line_start - 1) {
                if prev_char != '\n' && prev_char != '\r' {
                    return false;
                }
            }
        }

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

        while let Some(ch) = state.source().get_char_at(pos) {
            if ch == fence_char {
                fence_count += 1;
                pos += 1;
            }
            else {
                break;
            }
        }

        if fence_count < 3 {
            return false;
        }

        state.advance(fence_count);
        state.add_token(MarkdownTokenType::CodeFence, start_pos, state.get_position());

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
            state.add_token(MarkdownTokenType::CodeLanguage, lang_start, state.get_position());
        }

        true
    }

    /// Lexes blockquotes.
    pub fn lex_blockquote<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.source().get_char_at(check_pos) {
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
            state.add_token(MarkdownTokenType::BlockquoteMarker, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }

    /// Lexes horizontal rules.
    pub fn lex_horizontal_rule<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        let mut check_pos = start_pos;
        while check_pos > 0 {
            check_pos -= 1;
            if let Some(ch) = state.source().get_char_at(check_pos) {
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

                while let Some(current_ch) = state.source().get_char_at(pos) {
                    if current_ch == rule_char {
                        count += 1;
                        pos += 1
                    }
                    else if current_ch == ' ' || current_ch == '\t' {
                        pos += 1;
                    }
                    else {
                        break;
                    }
                }

                if count >= 3 {
                    while let Some(current_ch) = state.source().get_char_at(pos) {
                        if current_ch == '\n' || current_ch == '\r' {
                            break;
                        }
                        else if current_ch == ' ' || current_ch == '\t' {
                            pos += 1
                        }
                        else {
                            return false;
                        }
                    }

                    state.set_position(pos);
                    state.add_token(MarkdownTokenType::HorizontalRule, start_pos, state.get_position());
                    return true;
                }
            }
        }
        false
    }

    /// Lexes front matter.
    pub fn lex_front_matter<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if start_pos != 0 {
            return false;
        }

        if state.peek() == Some('-') && state.source().get_char_at(1) == Some('-') && state.source().get_char_at(2) == Some('-') {
            state.advance(3);
            let mut found_end = false;
            while state.not_at_end() {
                if state.peek() == Some('\n') || state.peek() == Some('\r') {
                    state.advance(1);
                    if state.peek() == Some('\n') {
                        state.advance(1)
                    }
                    if state.peek() == Some('-') && state.source().get_char_at(state.get_position() + 1) == Some('-') && state.source().get_char_at(state.get_position() + 2) == Some('-') {
                        state.advance(3);
                        found_end = true;
                        break;
                    }
                }
                else {
                    state.advance(1)
                }
            }

            if found_end {
                state.add_token(MarkdownTokenType::FrontMatter, start_pos, state.get_position());
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

    /// Handles indented code blocks.
    pub fn lex_indented_code_block<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if start_pos > 0 {
            if let Some(prev_char) = state.source().get_char_at(start_pos - 1) {
                if prev_char != '\n' && prev_char != '\r' {
                    return false;
                }
            }
        }

        let mut indent_count = 0;
        let mut pos = start_pos;
        while let Some(ch) = state.source().get_char_at(pos) {
            if ch == ' ' {
                indent_count += 1;
                pos += 1;
                if indent_count == 4 {
                    break;
                }
            }
            else if ch == '\t' {
                indent_count = 4;
                pos += 1;
                break;
            }
            else {
                break;
            }
        }

        if indent_count >= 4 {
            state.set_position(pos);
            state.add_token(MarkdownTokenType::CodeBlock, start_pos, state.get_position());
            true
        }
        else {
            false
        }
    }
}
