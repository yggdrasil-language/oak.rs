use crate::lexer::{MarkdownLexer, State, token_type::MarkdownTokenType};
use oak_core::Source;

/// Describes an opened fenced code block.
///
/// `container_depth` records how many blockquote markers preceded the fence on
/// its opening line, so the body can strip exactly that container prefix and
/// keep every other `>` byte as code content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FenceSpec {
    /// The fence character: `` ` `` or `~`.
    pub fence_char: char,
    /// The number of repeated fence characters on the opening line.
    pub fence_len: usize,
    /// The number of blockquote markers in front of the opening fence.
    pub container_depth: usize,
}

impl<'config> MarkdownLexer<'config> {
    /// Handles headings.
    pub fn lex_heading<S: Source + ?Sized>(&self, state: &mut State<S>) -> bool {
        let start_pos = state.get_position();

        if !is_heading_line_start(state, start_pos) {
            return false;
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

    /// Handles fenced code block openers.
    ///
    /// A fence may sit at the physical line start or behind a blockquote
    /// container prefix (`>` markers). Returns the fence description so the
    /// caller can keep the body opaque until the matching closing fence.
    pub fn lex_code_block<S: Source + ?Sized>(&self, state: &mut State<S>) -> Option<FenceSpec> {
        let start_pos = state.get_position();
        let container_depth = fence_line_prefix(state, start_pos)?;

        let fence_char = if let Some('`') = state.peek() {
            '`'
        }
        else if let Some('~') = state.peek() {
            '~'
        }
        else {
            return None;
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
            return None;
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

        Some(FenceSpec { fence_char, fence_len: fence_count, container_depth })
    }

    /// Lexes one physical line while a fenced block is open.
    ///
    /// The body is code, so only the container prefix keeps structural tokens.
    /// Every remaining byte of the line becomes one `CodeBlock` token. A line
    /// that closes the fence emits `CodeFence` and clears `fence`.
    pub fn lex_fence_body_line<S: Source + ?Sized>(&self, state: &mut State<S>, spec: &FenceSpec, fence: &mut Option<FenceSpec>) -> bool {
        let line_start = state.get_position();
        if !is_line_start(state, line_start) {
            return false;
        }

        let had_prefix = self.lex_fence_container_prefix(state, spec.container_depth);
        let content_start = state.get_position();

        // A closing fence allows up to three spaces, then a run of the opening
        // character at least as long as the opener, then whitespace only.
        let mut probe = content_start;
        let mut indent = 0;
        while indent < 4 && state.source().get_char_at(probe) == Some(' ') {
            probe += 1;
            indent += 1;
        }

        let closing_run = if indent < 4 {
            let mut run = 0;
            let mut cursor = probe;
            while state.source().get_char_at(cursor) == Some(spec.fence_char) {
                run += 1;
                cursor += 1;
            }
            if run >= spec.fence_len && rest_of_line_is_blank(state, cursor) { Some((probe, cursor)) } else { None }
        }
        else {
            None
        };

        if let Some((fence_start, fence_end)) = closing_run {
            if fence_start > content_start {
                state.set_position(fence_start);
                state.add_token(MarkdownTokenType::Whitespace, content_start, fence_start);
            }
            let fence_start = state.get_position();
            state.set_position(fence_end);
            state.add_token(MarkdownTokenType::CodeFence, fence_start, fence_end);
            *fence = None;
            return true;
        }

        let mut content_end = content_start;
        while let Some(ch) = state.source().get_char_at(content_end) {
            if ch == '\n' || ch == '\r' {
                break;
            }
            content_end += 1;
        }

        if content_end > content_start {
            state.set_position(content_end);
            state.add_token(MarkdownTokenType::CodeBlock, content_start, content_end);
            return true;
        }

        // An empty body line only keeps its prefix tokens. Without a prefix the
        // newline rule must still see the line start, so report false.
        had_prefix
    }

    /// Emits up to `depth` blockquote markers at the current line start.
    ///
    /// The prefix is bounded by the opening fence's container depth. A top-level
    /// fence therefore keeps a literal `>` at the start of a body line as code
    /// content instead of turning it into a `BlockquoteMarker`.
    fn lex_fence_container_prefix<S: Source + ?Sized>(&self, state: &mut State<S>, depth: usize) -> bool {
        let mut consumed = false;

        for _ in 0..depth {
            let mut cursor = state.get_position();
            let mut spaces = 0;
            while spaces < 3 && state.source().get_char_at(cursor) == Some(' ') {
                cursor += 1;
                spaces += 1;
            }
            if state.source().get_char_at(cursor) != Some('>') {
                break;
            }
            if cursor > state.get_position() {
                let ws_start = state.get_position();
                state.set_position(cursor);
                state.add_token(MarkdownTokenType::Whitespace, ws_start, cursor);
            }

            let marker_start = state.get_position();
            state.advance(1);
            state.add_token(MarkdownTokenType::BlockquoteMarker, marker_start, marker_start + 1);

            if matches!(state.peek(), Some(' ') | Some('\t')) {
                let ws_start = state.get_position();
                state.advance(1);
                state.add_token(MarkdownTokenType::Whitespace, ws_start, ws_start + 1);
            }
            consumed = true;
        }

        consumed
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

/// Returns `true` when `pos` can open an ATX heading.
///
/// A heading marker either starts the physical line or follows the blockquote prefix
/// (`>` markers plus optional spaces). Indentation of four or more spaces is an
/// indented code block, not a heading.
fn is_heading_line_start<S: Source + ?Sized>(state: &State<S>, pos: usize) -> bool {
    let mut index = pos;
    let mut indent = 0;

    while index > 0 {
        index -= 1;
        match state.source().get_char_at(index) {
            Some(' ') => indent += 1,
            Some('\t') => indent += 4,
            Some('>') => indent = 0,
            Some('\n') | Some('\r') => return indent < 4,
            _ => return false,
        }
    }

    indent < 4
}

/// Returns the blockquote depth when `pos` can open a fenced code block.
///
/// The position must sit at a physical line start or behind a blockquote
/// container prefix. Up to three spaces of indentation are allowed per level.
fn fence_line_prefix<S: Source + ?Sized>(state: &State<S>, pos: usize) -> Option<usize> {
    let mut index = pos;
    let mut indent = 0;
    let mut depth = 0;

    loop {
        while index > 0 {
            match state.source().get_char_at(index - 1) {
                Some(' ') => {
                    indent += 1;
                    index -= 1;
                }
                Some('\t') => {
                    indent += 4;
                    index -= 1;
                }
                _ => break,
            }
        }

        if indent >= 4 {
            return None;
        }

        if index == 0 {
            return Some(depth);
        }

        match state.source().get_char_at(index - 1) {
            Some('>') => {
                depth += 1;
                index -= 1;
                indent = 0;
            }
            Some('\n') | Some('\r') => return Some(depth),
            _ => return None,
        }
    }
}

/// Returns `true` when `pos` starts a physical line.
fn is_line_start<S: Source + ?Sized>(state: &State<S>, pos: usize) -> bool {
    pos == 0 || matches!(state.source().get_char_at(pos - 1), Some('\n') | Some('\r'))
}

/// Returns `true` when only spaces, tabs or the end of the line follow `pos`.
fn rest_of_line_is_blank<S: Source + ?Sized>(state: &State<S>, pos: usize) -> bool {
    let mut index = pos;
    while let Some(ch) = state.source().get_char_at(index) {
        match ch {
            ' ' | '\t' => index += 1,
            '\n' | '\r' => return true,
            _ => return false,
        }
    }
    true
}
