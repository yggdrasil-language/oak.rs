/// Element type definitions for the Notedown parser.
pub mod element_type;

use crate::{
    language::NotedownLanguage,
    lexer::{NotedownLexer, token_type::NoteTokenType},
    parser::element_type::NoteElementType,
};
use oak_core::{
    TextEdit, TokenType,
    errors::OakError,
    parser::{ParseCache, ParseOutput, Parser, ParserState},
    source::Source,
    tree::GreenNode,
};

pub(crate) type State<'a, S> = ParserState<'a, NotedownLanguage, S>;

/// Notedown parser implementation
pub struct NoteParser<'a> {
    /// Reference to the language configuration
    pub language: &'a NotedownLanguage,
}

impl<'a> NoteParser<'a> {
    /// Create a new parser with the given language configuration
    pub fn new(language: &'a NotedownLanguage) -> Self {
        Self { language }
    }
}

impl<'p> Parser<NotedownLanguage> for NoteParser<'p> {
    fn parse<'a, S: Source + ?Sized>(&self, source: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<NotedownLanguage>) -> ParseOutput<'a, NotedownLanguage> {
        let lexer = NotedownLexer::new(self.language);
        oak_core::parser::parse_with_lexer(&lexer, source, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            while state.not_at_end() {
                self.parse_block(state);
            }

            Ok(state.finish_at(checkpoint, NoteElementType::Root))
        })
    }
}

impl<'p> NoteParser<'p> {
    fn parse_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        while state.not_at_end() && (state.at(NoteTokenType::Newline) || state.at(NoteTokenType::Whitespace)) {
            state.bump();
        }
        if !state.not_at_end() {
            return;
        }

        let kind = state.peek_kind();
        match kind {
            Some(token) if heading_level_from_token(token).is_some() => self.parse_heading(state),
            Some(NoteTokenType::Hash) => self.parse_heading(state),
            Some(NoteTokenType::ListMarker) => self.parse_list_item(state),
            Some(NoteTokenType::Asterisk) | Some(NoteTokenType::Dash) | Some(NoteTokenType::Plus) => self.parse_list_item(state),
            Some(NoteTokenType::Pipe) => self.parse_table(state),
            Some(NoteTokenType::CodeFence) => self.parse_fenced_code_block(state),
            Some(NoteTokenType::Backtick) => self.parse_code_block(state),
            Some(NoteTokenType::BlockquoteMarker) => self.parse_blockquote(state),
            Some(NoteTokenType::HorizontalRule) => self.parse_horizontal_rule(state),
            _ => self.parse_paragraph(state),
        }
    }

    fn parse_heading<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        let level = if let Some(token) = state.peek_kind() {
            if let Some(level) = heading_level_from_token(token) {
                state.bump();
                level
            }
            else {
                let mut level = 0;
                while state.at(NoteTokenType::Hash) {
                    state.bump();
                    level += 1;
                }
                level
            }
        }
        else {
            0
        };

        self.parse_inline_content(state);
        if state.at(NoteTokenType::Newline) {
            state.bump();
        }

        let kind = match level {
            1..=6 => NoteElementType::Heading,
            _ => NoteElementType::Paragraph,
        };
        state.finish_at(checkpoint, kind);
    }

    fn parse_list_item<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // marker
        self.parse_inline_content(state);
        if state.at(NoteTokenType::Newline) {
            state.bump();
        }
        state.finish_at(checkpoint, NoteElementType::ListItem);
    }

    fn parse_table<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        while state.not_at_end() && state.at(NoteTokenType::Pipe) {
            self.parse_table_row(state);
            while state.at(NoteTokenType::Newline) || state.at(NoteTokenType::Whitespace) {
                state.bump();
            }
        }
        state.finish_at(checkpoint, NoteElementType::Table);
    }

    fn parse_table_row<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        while state.at(NoteTokenType::Pipe) {
            self.parse_table_cell(state);
        }
        if state.at(NoteTokenType::Newline) {
            state.bump();
        }
        state.finish_at(checkpoint, NoteElementType::TableRow);
    }

    fn parse_table_cell<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        state.bump(); // leading |
        let checkpoint = state.checkpoint();
        self.parse_inline_until(state, Some(NoteTokenType::Pipe));
        state.finish_at(checkpoint, NoteElementType::Root);
    }

    fn parse_paragraph<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        self.parse_inline_content(state);
        if state.at(NoteTokenType::Newline) {
            state.bump();
        }
        state.finish_at(checkpoint, NoteElementType::Paragraph);
    }

    fn parse_inline_content<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        self.parse_inline_until(state, None);
    }

    /// Parses inline content up to `Newline` or an optional `stop` token.
    ///
    /// 强调元素由一层显式栈维护：遇到定界符连串时用左右相邻字符判断开/合，
    /// 需要收尾就按“最内层优先”把连串字符分配给已打开的强调元素。
    fn parse_inline_until<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, stop: Option<NoteTokenType>) {
        // 已打开的强调元素：(起点, 元素类型, 定界符字符, 连串长度)
        let mut opens: Vec<((usize, usize), NoteElementType, char, usize)> = Vec::new();
        // 当前 token 之前（行首或前一个 token 之后）是否是空白，用于定界符 flanking 判断。
        let mut prev_blank = true;

        loop {
            if !state.not_at_end() || state.at(NoteTokenType::Newline) || stop.is_some_and(|kind| state.at(kind)) {
                break;
            }

            if state.at(NoteTokenType::Emphasis) {
                let ch = state.peek_text().and_then(|text| text.chars().next()).unwrap_or('*');

                // 连串由相邻的定界符 token 组成；跳过空白后，右边第一个原始 token
                // 是否为 trivia 决定右侧是否存在空白。
                let mut run_len = 0usize;
                while state.peek_kind_at(run_len) == Some(NoteTokenType::Emphasis) {
                    run_len += 1;
                }
                let right_token = state.peek_at(run_len);
                let right_blank = Self::blank_after_current(state, run_len);
                let right_punct = !right_blank && right_token.is_some_and(|token| Self::is_punctuation(token.kind));
                let can_open = prev_blank && !right_blank;
                let can_close = !prev_blank && (right_blank || right_punct);

                let matching = if can_close { opens.iter().rposition(|(_, _, open_char, _)| *open_char == ch) } else { None };

                if matching.is_some() {
                    let leftover = self.close_emphasis_run(state, &mut opens, ch, run_len);
                    for _ in 0..leftover {
                        state.bump();
                    }
                }
                else if can_open {
                    let element = if run_len >= 2 { NoteElementType::Token(NoteTokenType::Strong) } else { NoteElementType::Token(NoteTokenType::Emphasis) };
                    let start = state.checkpoint();
                    for _ in 0..run_len {
                        state.bump();
                    }
                    opens.push((start, element, ch, run_len));
                }
                else {
                    for _ in 0..run_len {
                        state.bump();
                    }
                }
                prev_blank = right_blank;
                continue;
            }

            if state.at(NoteTokenType::Strikethrough) {
                // `~~` 是单个不可分 token，按 GFM 的 flanking 规则决定开/合：
                // 左侧不是空白才能闭合，右侧不是空白才能开启。
                let right_blank = Self::blank_after_current(state, 1);
                let open = opens.iter().rposition(|(_, element, _, _)| matches!(element, NoteElementType::Token(NoteTokenType::Strikethrough)));
                if !prev_blank && open.is_some() {
                    state.bump();
                    let (start, element, _, _) = opens.remove(open.unwrap());
                    state.finish_at(start, element);
                }
                else if !right_blank {
                    let start = state.checkpoint();
                    state.bump();
                    opens.push((start, NoteElementType::Token(NoteTokenType::Strikethrough), '~', 1));
                }
                else {
                    state.bump();
                }
                prev_blank = right_blank;
                continue;
            }

            match state.peek_kind() {
                Some(NoteTokenType::Link) => {
                    self.parse_markdown_link(state, false);
                    prev_blank = false;
                }
                Some(NoteTokenType::Image) => {
                    self.parse_markdown_link(state, true);
                    prev_blank = false;
                }
                Some(NoteTokenType::LeftBracket) => {
                    self.parse_markdown_link(state, false);
                    prev_blank = false;
                }
                Some(NoteTokenType::InlineCode) => {
                    let start = state.checkpoint();
                    state.bump();
                    state.finish_at(start, NoteElementType::Token(NoteTokenType::InlineCode));
                    prev_blank = false;
                }
                _ => {
                    let blank_after = Self::blank_after_current(state, 1);
                    state.bump();
                    prev_blank = blank_after;
                }
            }
        }

        while let Some((start, element, _, _)) = opens.pop() {
            state.finish_at(start, element);
        }
    }

    /// 判断当前（或其后第 `offset` 个）原始 token 之后是否紧跟空白或换行。
    ///
    /// `advance`/`bump` 会自动跳过并吞掉 trivia，所以必须在消费 token 之前用
    /// 原始 token 流读取相邻空白，否则信息会丢失。`Newline` 不属于 trivia，
    /// 但对定界符 flanking 同样应视为空白。
    fn blank_after_current<'a, S: Source + ?Sized>(state: &State<'a, S>, offset: usize) -> bool {
        state.peek_at(offset).map(|token| token.kind.is_ignored() || token.kind == NoteTokenType::Newline).unwrap_or(true)
    }

    /// 是否为可用于定界符右侧 flanking 的标点 token。
    fn is_punctuation(kind: NoteTokenType) -> bool {
        matches!(
            kind,
            NoteTokenType::Dot
                | NoteTokenType::Colon
                | NoteTokenType::Exclamation
                | NoteTokenType::Pipe
                | NoteTokenType::RightParen
                | NoteTokenType::RightBracket
                | NoteTokenType::Dash
                | NoteTokenType::Plus
                | NoteTokenType::Tilde
                | NoteTokenType::Hash
                | NoteTokenType::LeftParen
                | NoteTokenType::LeftBracket
        )
    }

    /// 用一段定界符连串收尾同字符的已打开元素，最内层优先；返回未被消费的字符数。
    fn close_emphasis_run<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, opens: &mut Vec<((usize, usize), NoteElementType, char, usize)>, ch: char, run_len: usize) -> usize {
        let mut remaining = run_len;
        loop {
            let Some(pos) = opens.iter().rposition(|(_, _, open_char, _)| *open_char == ch)
            else {
                break;
            };
            // 先收尾位于其内侧（不同定界符字符）的元素，再处理匹配元素本身。
            while opens.len() > pos + 1 {
                let (inner_start, inner_element, _, _) = opens.pop().unwrap();
                state.finish_at(inner_start, inner_element);
            }
            let strength = opens[pos].3;
            if remaining < strength {
                break;
            }
            for _ in 0..strength {
                state.bump();
            }
            remaining -= strength;
            let (start, element, _, _) = opens.pop().unwrap();
            state.finish_at(start, element);
        }
        remaining
    }

    fn parse_code_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // ```
        while state.not_at_end() && !state.at(NoteTokenType::Backtick) {
            state.bump();
        }
        if state.at(NoteTokenType::Backtick) {
            state.bump();
        }
        state.finish_at(checkpoint, NoteElementType::CodeBlock);
    }

    fn parse_markdown_link<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>, is_image: bool) {
        let checkpoint = state.checkpoint();
        if !state.at(NoteTokenType::Link) && !state.at(NoteTokenType::Image) {
            state.bump(); // legacy `[` token
        }
        else {
            state.bump();
        }
        self.parse_inline_until(state, Some(NoteTokenType::RightBracket));
        if state.at(NoteTokenType::RightBracket) {
            state.bump();
        }
        if state.at(NoteTokenType::LeftParen) {
            state.bump();
            // 按括号深度配对，允许链接目标内出现成对括号，
            // 仅在深度回到 0 时才把该 `)` 当作链接目标收尾。
            let mut depth = 1usize;
            while state.not_at_end() && !state.at(NoteTokenType::Newline) {
                if state.at(NoteTokenType::LeftParen) {
                    depth += 1;
                    state.bump();
                }
                else if state.at(NoteTokenType::RightParen) {
                    depth -= 1;
                    state.bump();
                    if depth == 0 {
                        break;
                    }
                }
                else {
                    state.bump();
                }
            }
        }
        let kind = if is_image { NoteElementType::Image } else { NoteElementType::Link };
        state.finish_at(checkpoint, kind);
    }

    fn parse_blockquote<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        while state.at(NoteTokenType::BlockquoteMarker) {
            state.bump();
            while state.at(NoteTokenType::Whitespace) {
                state.bump();
            }
            self.parse_inline_content(state);
            if state.at(NoteTokenType::Newline) {
                state.bump();
            }
            while state.at(NoteTokenType::Whitespace) {
                state.bump();
            }
            if !state.at(NoteTokenType::BlockquoteMarker) {
                break;
            }
        }
        state.finish_at(checkpoint, NoteElementType::Blockquote);
    }

    fn parse_horizontal_rule<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump();
        if state.at(NoteTokenType::Newline) {
            state.bump();
        }
        state.finish_at(checkpoint, NoteElementType::HorizontalRule);
    }

    fn parse_fenced_code_block<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // opening fence
        if state.at(NoteTokenType::CodeLanguage) {
            state.bump();
        }
        while state.not_at_end() {
            if state.at(NoteTokenType::CodeFence) {
                state.bump();
                break;
            }
            state.bump();
        }
        while state.at(NoteTokenType::Newline) || state.at(NoteTokenType::Whitespace) {
            state.bump();
        }
        state.finish_at(checkpoint, NoteElementType::CodeBlock);
    }
}

fn heading_level_from_token(token: NoteTokenType) -> Option<u8> {
    match token {
        NoteTokenType::Heading1 => Some(1),
        NoteTokenType::Heading2 => Some(2),
        NoteTokenType::Heading3 => Some(3),
        NoteTokenType::Heading4 => Some(4),
        NoteTokenType::Heading5 => Some(5),
        NoteTokenType::Heading6 => Some(6),
        _ => None,
    }
}
