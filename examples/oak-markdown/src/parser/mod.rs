/// Element types for the Markdown language.
pub mod element_type;

use crate::{language::MarkdownLanguage, lexer::token_type::MarkdownTokenType, parser::element_type::MarkdownElementType as ET};
use oak_core::{Parser, ParserState, source::Source};

/// Parser for Markdown language.
pub struct MarkdownParser<'config> {
    pub(crate) config: &'config MarkdownLanguage,
}

impl<'config> MarkdownParser<'config> {
    /// Creates a new MarkdownParser with the given configuration.
    pub fn new(config: &'config MarkdownLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Parser<MarkdownLanguage> for MarkdownParser<'config> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[oak_core::TextEdit], cache: &'a mut impl oak_core::ParseCache<MarkdownLanguage>) -> oak_core::ParseOutput<'a, MarkdownLanguage> {
        let lexer = crate::lexer::MarkdownLexer::new(&self.config);
        oak_core::parser::parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();

            while state.not_at_end() {
                let item_checkpoint = state.checkpoint();
                if let Some(kind) = state.peek_kind() {
                    match kind {
                        MarkdownTokenType::FrontMatter => {
                            if self.config.allow_front_matter {
                                state.bump();
                                state.finish_at(item_checkpoint, ET::FrontMatter);
                            }
                            else {
                                self.parse_paragraph(state);
                            }
                        }
                        MarkdownTokenType::MathBlock => {
                            if self.config.allow_math {
                                state.bump();
                                state.finish_at(item_checkpoint, ET::MathBlock);
                            }
                            else {
                                self.parse_paragraph(state);
                            }
                        }
                        MarkdownTokenType::HtmlTag | MarkdownTokenType::HtmlComment => {
                            if self.config.allow_html {
                                state.bump();
                                state.finish_at(item_checkpoint, ET::from(kind));
                            }
                            else {
                                self.parse_paragraph(state);
                            }
                        }
                        MarkdownTokenType::XmlTag | MarkdownTokenType::XmlComment => {
                            if self.config.allow_xml {
                                state.bump();
                                state.finish_at(item_checkpoint, ET::from(kind));
                            }
                            else {
                                self.parse_paragraph(state);
                            }
                        }
                        MarkdownTokenType::FootnoteDefinition => {
                            state.bump();
                            self.parse_inlines_until_newline(state);
                            state.finish_at(item_checkpoint, ET::FootnoteDefinition);
                        }
                        MarkdownTokenType::DefinitionDescription => {
                            if self.config.allow_definition_lists {
                                let dl_checkpoint = item_checkpoint;
                                // Parse definition description
                                state.bump();
                                self.parse_inlines_until_newline(state);
                                state.finish_at(dl_checkpoint, ET::DefinitionList);
                            }
                            else {
                                self.parse_paragraph(state);
                            }
                        }
                        MarkdownTokenType::Abbreviation => {
                            if self.config.allow_abbreviations {
                                state.bump();
                                self.parse_inlines_until_newline(state);
                                state.finish_at(item_checkpoint, ET::Abbreviation);
                            }
                            else {
                                self.parse_paragraph(state);
                            }
                        }
                        MarkdownTokenType::Heading1 | MarkdownTokenType::Heading2 | MarkdownTokenType::Heading3 | MarkdownTokenType::Heading4 | MarkdownTokenType::Heading5 | MarkdownTokenType::Heading6 => {
                            state.bump();
                            self.parse_inlines_until_newline(state);
                            state.finish_at(item_checkpoint, ET::from(kind));
                        }
                        MarkdownTokenType::ListMarker => {
                            let list_checkpoint = item_checkpoint;
                            let mut is_ordered = false;
                            if let Some(text) = state.peek_text() {
                                if text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                                    is_ordered = true;
                                }
                            }

                            while state.not_at_end() {
                                if let Some(MarkdownTokenType::ListMarker) = state.peek_kind() {
                                    let current_is_ordered = if let Some(text) = state.peek_text() { text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) } else { false };

                                    if current_is_ordered != is_ordered && state.checkpoint() != list_checkpoint {
                                        break;
                                    }

                                    let li_checkpoint = state.checkpoint();
                                    state.bump(); // Marker

                                    // 解析列表项内容
                                    self.parse_inlines_until_newline(state);

                                    // 检查是否有嵌套列表
                                    if let Some(MarkdownTokenType::Newline) = state.peek_kind() {
                                        let nl_checkpoint = state.checkpoint();
                                        state.bump();

                                        // 检查是否有缩进的嵌套列表
                                        let mut indent_level = 0;
                                        while state.not_at_end() {
                                            if let Some(MarkdownTokenType::Whitespace) = state.peek_kind() {
                                                state.bump();
                                                indent_level += 1;
                                            }
                                            else if let Some(MarkdownTokenType::ListMarker) = state.peek_kind() {
                                                // 递归解析嵌套列表
                                                let nested_list_checkpoint = state.checkpoint();
                                                let nested_is_ordered = if let Some(text) = state.peek_text() { text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) } else { false };

                                                while state.not_at_end() {
                                                    if let Some(MarkdownTokenType::ListMarker) = state.peek_kind() {
                                                        let nested_current_is_ordered = if let Some(text) = state.peek_text() { text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) } else { false };

                                                        if nested_current_is_ordered != nested_is_ordered && state.checkpoint() != nested_list_checkpoint {
                                                            break;
                                                        }

                                                        let nested_li_checkpoint = state.checkpoint();
                                                        state.bump(); // Marker
                                                        self.parse_inlines_until_newline(state);
                                                        state.finish_at(nested_li_checkpoint, ET::ListItem);

                                                        if let Some(MarkdownTokenType::Newline) = state.peek_kind() {
                                                            let nested_nl_checkpoint = state.checkpoint();
                                                            state.bump();
                                                            let mut nested_indent_level = 0;
                                                            while state.not_at_end() {
                                                                if let Some(MarkdownTokenType::Whitespace) = state.peek_kind() {
                                                                    state.bump();
                                                                    nested_indent_level += 1;
                                                                }
                                                                else {
                                                                    break;
                                                                }
                                                            }
                                                            if nested_indent_level <= indent_level || !matches!(state.peek_kind(), Some(MarkdownTokenType::ListMarker)) {
                                                                state.restore(nested_nl_checkpoint);
                                                                break;
                                                            }
                                                        }
                                                        else {
                                                            break;
                                                        }
                                                    }
                                                    else {
                                                        break;
                                                    }
                                                }
                                                state.finish_at(nested_list_checkpoint, ET::List);
                                                break;
                                            }
                                            else {
                                                state.restore(nl_checkpoint);
                                                break;
                                            }
                                        }
                                    }

                                    state.finish_at(li_checkpoint, ET::ListItem);

                                    if let Some(MarkdownTokenType::Newline) = state.peek_kind() {
                                        let nl_checkpoint = state.checkpoint();
                                        state.bump();
                                        if !matches!(state.peek_kind(), Some(MarkdownTokenType::ListMarker)) {
                                            state.restore(nl_checkpoint);
                                            break;
                                        }
                                        let next_is_ordered = if let Some(text) = state.peek_text() { text.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false) } else { false };
                                        if next_is_ordered != is_ordered {
                                            state.restore(nl_checkpoint);
                                            break;
                                        }
                                    }
                                    else {
                                        break;
                                    }
                                }
                                else {
                                    break;
                                }
                            }
                            state.finish_at(list_checkpoint, ET::List);
                        }
                        MarkdownTokenType::BlockquoteMarker => {
                            let blockquote_checkpoint = item_checkpoint;
                            state.bump();

                            while state.not_at_end() {
                                if let Some(next_kind) = state.peek_kind() {
                                    if next_kind == MarkdownTokenType::Newline {
                                        state.bump();
                                        if let Some(after_nl) = state.peek_kind() {
                                            if after_nl == MarkdownTokenType::BlockquoteMarker {
                                                // 处理嵌套引用
                                                let nested_quote_checkpoint = state.checkpoint();
                                                state.bump();

                                                while state.not_at_end() {
                                                    if let Some(nested_next_kind) = state.peek_kind() {
                                                        if nested_next_kind == MarkdownTokenType::Newline {
                                                            state.bump();
                                                            if let Some(nested_after_nl) = state.peek_kind() {
                                                                if nested_after_nl == MarkdownTokenType::BlockquoteMarker {
                                                                    // 递归处理更深层的嵌套引用
                                                                    let deeper_nested_quote_checkpoint = state.checkpoint();
                                                                    state.bump();

                                                                    while state.not_at_end() {
                                                                        if let Some(deeper_nested_next_kind) = state.peek_kind() {
                                                                            if deeper_nested_next_kind == MarkdownTokenType::Newline {
                                                                                state.bump();
                                                                                if let Some(deeper_nested_after_nl) = state.peek_kind() {
                                                                                    if deeper_nested_after_nl == MarkdownTokenType::BlockquoteMarker {
                                                                                        state.bump();
                                                                                        continue;
                                                                                    }
                                                                                    if deeper_nested_after_nl != MarkdownTokenType::Whitespace && deeper_nested_after_nl != MarkdownTokenType::Text {
                                                                                        break;
                                                                                    }
                                                                                }
                                                                                else {
                                                                                    break;
                                                                                }
                                                                            }
                                                                            else if self.is_block_start(deeper_nested_next_kind) && deeper_nested_next_kind != MarkdownTokenType::BlockquoteMarker {
                                                                                break;
                                                                            }
                                                                        }
                                                                        self.parse_inline(state);
                                                                    }

                                                                    state.finish_at(deeper_nested_quote_checkpoint, ET::Blockquote);
                                                                    continue;
                                                                }
                                                                if nested_after_nl != MarkdownTokenType::Whitespace && nested_after_nl != MarkdownTokenType::Text {
                                                                    break;
                                                                }
                                                            }
                                                            else {
                                                                break;
                                                            }
                                                        }
                                                        else if self.is_block_start(nested_next_kind) && nested_next_kind != MarkdownTokenType::BlockquoteMarker {
                                                            break;
                                                        }
                                                    }
                                                    self.parse_inline(state);
                                                }

                                                state.finish_at(nested_quote_checkpoint, ET::Blockquote);
                                                continue;
                                            }
                                            if after_nl != MarkdownTokenType::Whitespace && after_nl != MarkdownTokenType::Text {
                                                break;
                                            }
                                        }
                                        else {
                                            break;
                                        }
                                    }
                                    else if self.is_block_start(next_kind) && next_kind != MarkdownTokenType::BlockquoteMarker {
                                        break;
                                    }
                                }
                                self.parse_inline(state);
                            }

                            state.finish_at(blockquote_checkpoint, ET::Blockquote);
                        }
                        MarkdownTokenType::CodeFence => {
                            state.bump();
                            if let Some(MarkdownTokenType::CodeLanguage) = state.peek_kind() {
                                state.bump();
                            }
                            while state.not_at_end() {
                                if let Some(next_kind) = state.peek_kind() {
                                    if next_kind == MarkdownTokenType::CodeFence {
                                        state.bump();
                                        break;
                                    }
                                }
                                state.bump();
                            }
                            state.finish_at(item_checkpoint, ET::CodeBlock);
                        }
                        MarkdownTokenType::HorizontalRule => {
                            state.bump();
                            state.finish_at(item_checkpoint, ET::HorizontalRule);
                        }
                        MarkdownTokenType::Pipe => {
                            let table_checkpoint = item_checkpoint;
                            state.bump(); // 跳过第一个管道
                            // 解析表格行
                            while state.not_at_end() {
                                let row_checkpoint = state.checkpoint();
                                // 解析行内容和单元格
                                while state.not_at_end() {
                                    if let Some(next_kind) = state.peek_kind() {
                                        if next_kind == MarkdownTokenType::Newline {
                                            break;
                                        }
                                        else if next_kind == MarkdownTokenType::Pipe {
                                            state.bump(); // 跳过管道
                                        }
                                    }
                                    let cell_checkpoint = state.checkpoint();
                                    // 解析单元格内容
                                    while state.not_at_end() {
                                        if let Some(next_kind) = state.peek_kind() {
                                            if next_kind == MarkdownTokenType::Pipe || next_kind == MarkdownTokenType::Newline {
                                                break;
                                            }
                                        }
                                        self.parse_inline(state);
                                    }
                                    state.finish_at(cell_checkpoint, ET::TableCell);
                                }
                                state.finish_at(row_checkpoint, ET::TableRow);

                                if let Some(MarkdownTokenType::Newline) = state.peek_kind() {
                                    let checkpoint_before_nl = state.checkpoint();
                                    state.bump();
                                    let mut is_table_line = false;
                                    while state.not_at_end() {
                                        if let Some(kind) = state.peek_kind() {
                                            if kind == MarkdownTokenType::Whitespace {
                                                state.bump();
                                            }
                                            else if kind == MarkdownTokenType::Pipe {
                                                is_table_line = true;
                                                break;
                                            }
                                            else if kind == MarkdownTokenType::Dash || kind == MarkdownTokenType::Colon {
                                                // 处理表格分隔线
                                                let separator_checkpoint = state.checkpoint();
                                                while state.not_at_end() {
                                                    if let Some(sep_kind) = state.peek_kind() {
                                                        if sep_kind == MarkdownTokenType::Newline {
                                                            break;
                                                        }
                                                    }
                                                    state.bump();
                                                }
                                                state.finish_at(separator_checkpoint, ET::TableSeparator);
                                                break;
                                            }
                                            else {
                                                break;
                                            }
                                        }
                                        else {
                                            break;
                                        }
                                    }
                                    if is_table_line {
                                        state.bump(); // 跳过新行的管道
                                        continue;
                                    }
                                    else {
                                        state.restore(checkpoint_before_nl);
                                        break;
                                    }
                                }
                                else {
                                    break;
                                }
                            }
                            state.finish_at(table_checkpoint, ET::Table);
                        }
                        MarkdownTokenType::Newline | MarkdownTokenType::Whitespace => {
                            state.bump();
                        }
                        _ => {
                            self.parse_paragraph(state);
                        }
                    }
                }
                else {
                    state.advance();
                }
            }

            let root = state.finish_at(checkpoint, ET::Root);
            Ok(root)
        })
    }
}

impl<'config> MarkdownParser<'config> {
    fn is_opening_bracket(kind: Option<MarkdownTokenType>, text: Option<&str>) -> bool {
        kind == Some(MarkdownTokenType::LBracket) || text == Some("[")
    }

    fn is_closing_bracket(kind: Option<MarkdownTokenType>, text: Option<&str>) -> bool {
        kind == Some(MarkdownTokenType::RBracket) || text == Some("]")
    }

    fn is_opening_paren(kind: Option<MarkdownTokenType>, text: Option<&str>) -> bool {
        kind == Some(MarkdownTokenType::LParen) || text == Some("(")
    }

    fn is_closing_paren(kind: Option<MarkdownTokenType>, text: Option<&str>) -> bool {
        kind == Some(MarkdownTokenType::RParen) || text == Some(")")
    }

    fn bump_if_closing_bracket<'a, S: Source + ?Sized>(state: &mut ParserState<'a, MarkdownLanguage, S>) -> bool {
        let kind = state.peek_kind();
        let text = state.peek_text();
        if Self::is_closing_bracket(kind, text.as_deref()) {
            state.bump();
            true
        }
        else {
            false
        }
    }

    fn bump_if_opening_bracket<'a, S: Source + ?Sized>(state: &mut ParserState<'a, MarkdownLanguage, S>) -> bool {
        let kind = state.peek_kind();
        let text = state.peek_text();
        if Self::is_opening_bracket(kind, text.as_deref()) {
            state.bump();
            true
        }
        else {
            false
        }
    }

    fn bump_if_closing_paren<'a, S: Source + ?Sized>(state: &mut ParserState<'a, MarkdownLanguage, S>) -> bool {
        let kind = state.peek_kind();
        let text = state.peek_text();
        if Self::is_closing_paren(kind, text.as_deref()) {
            state.bump();
            true
        }
        else {
            false
        }
    }

    fn is_block_start(&self, kind: MarkdownTokenType) -> bool {
        matches!(
            kind,
            MarkdownTokenType::Heading1
                | MarkdownTokenType::Heading2
                | MarkdownTokenType::Heading3
                | MarkdownTokenType::Heading4
                | MarkdownTokenType::Heading5
                | MarkdownTokenType::Heading6
                | MarkdownTokenType::BlockquoteMarker
                | MarkdownTokenType::CodeFence
                | MarkdownTokenType::ListMarker
                | MarkdownTokenType::HorizontalRule
                | MarkdownTokenType::MathBlock
                | MarkdownTokenType::FrontMatter
                | MarkdownTokenType::FootnoteDefinition
                | MarkdownTokenType::DefinitionDescription
                | MarkdownTokenType::Abbreviation
        )
    }

    fn parse_paragraph<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, MarkdownLanguage, S>) {
        let checkpoint = state.checkpoint();
        while state.not_at_end() {
            if let Some(next_kind) = state.peek_kind() {
                if next_kind == MarkdownTokenType::Newline {
                    let cp = state.checkpoint();
                    state.bump();
                    if let Some(after_nl) = state.peek_kind() {
                        if after_nl == MarkdownTokenType::Newline || self.is_block_start(after_nl) {
                            state.restore(cp);
                            break;
                        }
                    }
                    else {
                        break;
                    }
                }
                else if self.is_block_start(next_kind) {
                    break;
                }
            }
            self.parse_inline(state);
        }
        state.finish_at(checkpoint, ET::Paragraph);
    }

    fn parse_inlines_until_newline<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, MarkdownLanguage, S>) {
        while state.not_at_end() {
            if let Some(kind) = state.peek_kind() {
                if kind == MarkdownTokenType::Newline {
                    break;
                }
            }
            self.parse_inline(state);
        }
    }

    fn parse_inline<'a, S: Source + ?Sized>(&self, state: &mut ParserState<'a, MarkdownLanguage, S>) {
        let checkpoint = state.checkpoint();
        if let Some(kind) = state.peek_kind() {
            match kind {
                MarkdownTokenType::Emphasis | MarkdownTokenType::Strong | MarkdownTokenType::Strikethrough => {
                    let marker_kind = kind;
                    state.bump(); // Start marker
                    while state.not_at_end() && state.peek_kind() != Some(marker_kind) && state.peek_kind() != Some(MarkdownTokenType::Newline) {
                        self.parse_inline(state);
                    }
                    if state.peek_kind() == Some(marker_kind) {
                        state.bump(); // End marker
                    }
                    state.finish_at(checkpoint, ET::from(marker_kind));
                }
                MarkdownTokenType::Link | MarkdownTokenType::Image => {
                    let is_image = kind == MarkdownTokenType::Image;
                    state.bump();
                    while state.not_at_end() && state.peek_kind() != Some(MarkdownTokenType::Newline) {
                        let text = state.peek_text();
                        if Self::is_closing_bracket(state.peek_kind(), text.as_deref()) {
                            break;
                        }
                        self.parse_inline(state);
                    }
                    Self::bump_if_closing_bracket(state);
                    {
                        let text = state.peek_text();
                        if Self::is_opening_bracket(state.peek_kind(), text.as_deref()) {
                            state.bump();
                            while state.not_at_end() && state.peek_kind() != Some(MarkdownTokenType::Newline) {
                                let text = state.peek_text();
                                if Self::is_closing_bracket(state.peek_kind(), text.as_deref()) {
                                    break;
                                }
                                state.bump();
                            }
                            Self::bump_if_closing_bracket(state);
                        }
                        else {
                            let text = state.peek_text();
                            if Self::is_opening_paren(state.peek_kind(), text.as_deref()) {
                                state.bump();
                                while state.not_at_end() && state.peek_kind() != Some(MarkdownTokenType::Newline) {
                                    let text = state.peek_text();
                                    if Self::is_closing_paren(state.peek_kind(), text.as_deref()) {
                                        break;
                                    }
                                    state.bump();
                                }
                                Self::bump_if_closing_paren(state);
                            }
                        }
                    }
                    state.finish_at(checkpoint, if is_image { ET::Image } else { ET::Link });
                }
                MarkdownTokenType::InlineCode => {
                    state.bump();
                    state.finish_at(checkpoint, ET::InlineCode);
                }
                MarkdownTokenType::MathInline | MarkdownTokenType::MathBlock => {
                    state.bump();
                    state.finish_at(checkpoint, ET::from(kind));
                }
                MarkdownTokenType::Superscript | MarkdownTokenType::Subscript => {
                    let marker_kind = kind;
                    state.bump(); // Start marker
                    while state.not_at_end() && state.peek_kind() != Some(marker_kind) && state.peek_kind() != Some(MarkdownTokenType::Newline) {
                        self.parse_inline(state);
                    }
                    if state.peek_kind() == Some(marker_kind) {
                        state.bump(); // End marker
                    }
                    state.finish_at(checkpoint, ET::from(marker_kind));
                }
                MarkdownTokenType::FootnoteReference | MarkdownTokenType::FootnoteDefinition => {
                    state.bump();
                    state.finish_at(checkpoint, ET::from(kind));
                }
                MarkdownTokenType::TaskMarker => {
                    state.bump(); // [ ] or [x]
                    state.finish_at(checkpoint, ET::TaskMarker);
                }
                MarkdownTokenType::AutoLink => {
                    state.bump();
                    state.finish_at(checkpoint, ET::AutoLink);
                }
                _ => {
                    state.bump();
                }
            }
        }
        else {
            state.advance();
        }
    }
}
