/// Element type definitions for the Notedown parser.
pub mod element_type;

use crate::{
    language::NotedownLanguage,
    lexer::{NotedownLexer, token_type::NoteTokenType},
    parser::element_type::NoteElementType,
};
use oak_core::{
    TextEdit,
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
        while state.not_at_end()
            && (state.at(NoteTokenType::Newline) || state.at(NoteTokenType::Whitespace))
        {
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
            Some(NoteTokenType::Asterisk) | Some(NoteTokenType::Dash) | Some(NoteTokenType::Plus) => {
                self.parse_list_item(state)
            }
            Some(NoteTokenType::Pipe) => self.parse_table(state),
            Some(NoteTokenType::CodeFence) => self.parse_fenced_code_block(state),
            Some(NoteTokenType::Backtick) => self.parse_code_block(state),
            Some(NoteTokenType::BlockquoteMarker) => self.parse_blockquote(state),
            _ => self.parse_paragraph(state),
        }
    }

    fn parse_heading<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        let level = if let Some(token) = state.peek_kind() {
            if let Some(level) = heading_level_from_token(token) {
                state.bump();
                level
            } else {
                let mut level = 0;
                while state.at(NoteTokenType::Hash) {
                    state.bump();
                    level += 1;
                }
                level
            }
        } else {
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
        while state.not_at_end() && !state.at(NoteTokenType::Pipe) && !state.at(NoteTokenType::Newline) {
            self.parse_table_cell_inline(state);
        }
        state.finish_at(checkpoint, NoteElementType::Root);
    }

    fn parse_table_cell_inline<'a, S: Source + ?Sized>(&self, state: &mut State<'a, S>) {
        let checkpoint = state.checkpoint();
        let kind = state.peek_kind();
        match kind {
            Some(NoteTokenType::Asterisk) | Some(NoteTokenType::Underscore) => {
                let marker = kind.unwrap();
                state.bump();
                while state.not_at_end()
                    && !state.at(marker)
                    && !state.at(NoteTokenType::Newline)
                    && !state.at(NoteTokenType::Pipe)
                {
                    self.parse_table_cell_inline(state);
                }
                if state.at(marker) {
                    state.bump();
                }
                state.finish_at(checkpoint, NoteElementType::Root);
            }
            Some(NoteTokenType::Link) => self.parse_markdown_link(state, false),
            Some(NoteTokenType::Image) => self.parse_markdown_link(state, true),
            Some(NoteTokenType::LeftBracket) => self.parse_markdown_link(state, false),
            _ => {
                state.bump();
            }
        }
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
        while state.not_at_end() && !state.at(NoteTokenType::Newline) {
            let checkpoint = state.checkpoint();
            let kind = state.peek_kind();
            match kind {
                Some(NoteTokenType::Asterisk) | Some(NoteTokenType::Underscore) => {
                    let marker = kind.unwrap();
                    state.bump();
                    while state.not_at_end() && !state.at(marker) && !state.at(NoteTokenType::Newline) {
                        self.parse_inline_content(state);
                    }
                    if state.at(marker) {
                        state.bump();
                    }
                    state.finish_at(checkpoint, NoteElementType::Root);
                }
                Some(NoteTokenType::Link) => {
                    self.parse_markdown_link(state, false);
                }
                Some(NoteTokenType::Image) => {
                    self.parse_markdown_link(state, true);
                }
                Some(NoteTokenType::LeftBracket) => {
                    self.parse_markdown_link(state, false);
                }
                _ => {
                    state.bump();
                }
            }
        }
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
        } else {
            state.bump();
        }
        while state.not_at_end()
            && !state.at(NoteTokenType::RightBracket)
            && !state.at(NoteTokenType::Newline)
        {
            self.parse_inline_content(state);
        }
        if state.at(NoteTokenType::RightBracket) {
            state.bump();
        }
        if state.at(NoteTokenType::LeftParen) {
            state.bump();
            while state.not_at_end()
                && !state.at(NoteTokenType::RightParen)
                && !state.at(NoteTokenType::Newline)
            {
                state.bump();
            }
            if state.at(NoteTokenType::RightParen) {
                state.bump();
            }
        }
        let kind = if is_image {
            NoteElementType::Image
        } else {
            NoteElementType::Link
        };
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
