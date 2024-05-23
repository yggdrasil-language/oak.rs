/// Element types for the Mermaid language.
pub mod element_type;

use crate::{
    language::MermaidLanguage,
    lexer::{MermaidLexer, token_type::MermaidTokenType},
    parser::element_type::MermaidElementType,
};
use oak_core::{
    OakError, TextEdit,
    parser::{ParseCache, ParseOutput, Parser, ParserState},
    source::Source,
};

pub(crate) type State<'a, S> = ParserState<'a, MermaidLanguage, S>;

/// `subgraph`、`direction`、样式声明等当前不支持、需要整行跳过的语句。
///
/// `subgraph` 与 `end` 由 [`MermaidParser::parse_subgraph`] 处理，不在此列表中。
const SKIPPED_KEYWORDS: &[&str] = &["direction", "style", "classDef", "class", "linkStyle", "click", "accTitle", "accDescr"];

/// 其他 Mermaid 图表头。遇到它们时结束当前 flowchart 段落，避免把
/// 序列图/类图的内容误当成 flowchart 节点。
const DIAGRAM_HEADERS: &[&str] = &[
    "sequenceDiagram",
    "classDiagram",
    "stateDiagram-v2",
    "stateDiagram",
    "erDiagram",
    "gantt",
    "pie",
    "gitGraph",
    "journey",
    "mindmap",
    "requirementDiagram",
    "block-beta",
    "sankey-beta",
    "xychart-beta",
    "quadrantChart",
    "timeline",
    "zenuml",
    "C4Context",
];

/// Mermaid parser implementation.
pub struct MermaidParser<'a> {
    pub(crate) _language: &'a MermaidLanguage,
}

impl<'a> MermaidParser<'a> {
    /// Creates a new `MermaidParser`.
    pub fn new(language: &'a MermaidLanguage) -> Self {
        Self { _language: language }
    }

    /// 逐行扫描源码，只把 flowchart 段落包成 `Graph` 元素。
    fn parse_root<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> Result<(), OakError> {
        while state.not_at_end() {
            match state.peek_kind() {
                Some(MermaidTokenType::Newline) => state.bump(),
                Some(MermaidTokenType::Graph) => self.parse_graph(state),
                Some(MermaidTokenType::Eof) => {
                    state.bump();
                    break;
                }
                _ => self.skip_line(state),
            }
        }
        Ok(())
    }

    /// 解析 `graph`/`flowchart` 段落：头部（关键字 + 方向）以及后续每一行。
    fn parse_graph<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) {
        let checkpoint = state.checkpoint();
        state.bump(); // `graph` / `flowchart`
        if state.at(MermaidTokenType::Direction) {
            state.bump();
        }
        self.skip_line(state);

        while state.not_at_end() {
            if state.at(MermaidTokenType::Graph) || state.at(MermaidTokenType::Eof) {
                break;
            }
            if state.at(MermaidTokenType::Newline) {
                state.bump();
                continue;
            }
            if Self::starts_with_keyword(state, DIAGRAM_HEADERS) {
                break;
            }
            if Self::starts_with_keyword(state, &["subgraph"]) {
                if !self.parse_subgraph(state) {
                    self.skip_line(state);
                }
                continue;
            }
            if Self::starts_with_keyword(state, SKIPPED_KEYWORDS) || Self::starts_with_keyword(state, &["end"]) {
                self.skip_line(state);
                continue;
            }

            let line_checkpoint = state.checkpoint();
            if self.parse_line(state) {
                if state.at(MermaidTokenType::Newline) {
                    state.bump();
                }
            }
            else {
                state.restore(line_checkpoint);
                self.skip_line(state);
            }
        }

        state.finish_at(checkpoint, MermaidElementType::Graph);
    }

    /// 解析 `subgraph ... end` 块，支持嵌套。
    ///
    /// 头部行（`subgraph title` 或 `subgraph id[title]`）整行跳过，块内每一行
    /// 按 flowchart 规则解析，遇到 `end` 结束。未闭合的块在段落末尾结束。
    fn parse_subgraph<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let checkpoint = state.checkpoint();
        state.bump(); // `subgraph`
        self.skip_line(state);

        while state.not_at_end() {
            if state.at(MermaidTokenType::Eof) {
                break;
            }
            if state.at(MermaidTokenType::Newline) {
                state.bump();
                continue;
            }
            if state.at(MermaidTokenType::Graph) || Self::starts_with_keyword(state, DIAGRAM_HEADERS) {
                break;
            }
            if Self::starts_with_keyword(state, &["end"]) {
                state.bump();
                self.skip_line(state);
                state.finish_at(checkpoint, MermaidElementType::Subgraph);
                return true;
            }
            if Self::starts_with_keyword(state, &["subgraph"]) {
                if !self.parse_subgraph(state) {
                    self.skip_line(state);
                }
                continue;
            }
            if Self::starts_with_keyword(state, SKIPPED_KEYWORDS) {
                self.skip_line(state);
                continue;
            }

            let line_checkpoint = state.checkpoint();
            if self.parse_line(state) {
                if state.at(MermaidTokenType::Newline) {
                    state.bump();
                }
            }
            else {
                state.restore(line_checkpoint);
                self.skip_line(state);
            }
        }

        state.finish_at(checkpoint, MermaidElementType::Subgraph);
        true
    }

    /// 解析一行 flowchart 语句，产出 `Node` 或 `Edge` 元素。
    ///
    /// 成功时当前 token 停在行尾（`Newline` 或 `Eof`），由调用方消费换行。
    /// 失败时调用方负责回滚到行首并整行跳过。
    fn parse_line<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        if !state.at(MermaidTokenType::Id) {
            return false;
        }

        let checkpoint = state.checkpoint();
        state.bump();
        if state.at(MermaidTokenType::Label) {
            state.bump();
        }

        if state.at(MermaidTokenType::Arrow) {
            loop {
                if !self.parse_arrow(state) || !state.at(MermaidTokenType::Id) {
                    return false;
                }
                state.bump();
                if state.at(MermaidTokenType::Label) {
                    state.bump();
                }
                if !state.at(MermaidTokenType::Arrow) {
                    break;
                }
            }
            if !Self::at_line_end(state) {
                return false;
            }
            state.finish_at(checkpoint, MermaidElementType::Edge);
        }
        else {
            if !Self::at_line_end(state) {
                return false;
            }
            state.finish_at(checkpoint, MermaidElementType::Node);
        }
        true
    }

    /// 解析一个箭头（含边标签），支持 `-- text -->` 的两段式写法。
    fn parse_arrow<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> bool {
        let text = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
        state.bump();

        if text == "--" {
            // 起始 `--` 与结束箭头之间的文本由 builder 从 green tree 还原。
            while state.not_at_end() && !state.at(MermaidTokenType::Arrow) && !state.at(MermaidTokenType::Newline) && !state.at(MermaidTokenType::Eof) {
                state.bump();
            }
            if !state.at(MermaidTokenType::Arrow) {
                return false;
            }
            state.bump();
        }
        else if state.at(MermaidTokenType::Label) {
            // `|text|` 形式的边标签。
            state.bump();
        }
        true
    }

    /// 跳过当前行剩余 token（包含行尾换行）。
    fn skip_line<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) {
        while state.not_at_end() && !state.at(MermaidTokenType::Newline) && !state.at(MermaidTokenType::Eof) {
            state.bump();
        }
        if state.at(MermaidTokenType::Newline) {
            state.bump();
        }
    }

    /// 当前 token 是否为给定关键字之一。
    fn starts_with_keyword<S: Source + ?Sized>(state: &State<'_, S>, keywords: &[&str]) -> bool {
        matches!(state.peek_kind(), Some(MermaidTokenType::Id)) && state.peek_text().is_some_and(|text| keywords.contains(&text.as_ref()))
    }

    /// 当前是否停在行尾。
    fn at_line_end<S: Source + ?Sized>(state: &State<'_, S>) -> bool {
        state.at(MermaidTokenType::Newline) || state.at(MermaidTokenType::Eof)
    }
}

impl<'p> Parser<MermaidLanguage> for MermaidParser<'p> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<MermaidLanguage>) -> ParseOutput<'a, MermaidLanguage> {
        let lexer = MermaidLexer::new(self._language);
        oak_core::parser::parse_with_lexer(&lexer, text, edits, cache, |state| {
            let checkpoint = state.checkpoint();
            self.parse_root(state)?;
            Ok(state.finish_at(checkpoint, MermaidElementType::Root))
        })
    }
}
