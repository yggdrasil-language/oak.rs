/// Token types for the Mermaid language.
pub mod token_type;

use crate::{language::MermaidLanguage, lexer::token_type::MermaidTokenType};
use oak_core::{
    Lexer, LexerState,
    lexer::{LexOutput, LexerCache},
    source::{Source, TextEdit},
};

type State<'s, S> = LexerState<'s, S, MermaidLanguage>;

/// 按最长匹配顺序排列的箭头片段。
///
/// `--` 是 `-->`、`---` 的前缀，必须放在最后；`-- text -->` 会被拆成
/// 起始 `--` 与结束箭头两个 token，由 parser 负责合并。
const ARROWS: &[&str] = &["-.->", "-->", "==>", "---", "--"];

/// 合法的 flowchart 方向。
const DIRECTIONS: &[&str] = &["TD", "TB", "BT", "LR", "RL"];

/// A lexer for the Mermaid language.
#[derive(Clone)]
pub struct MermaidLexer<'config> {
    config: &'config MermaidLanguage,
}

impl<'config> Lexer<MermaidLanguage> for MermaidLexer<'config> {
    fn lex<'a, S: Source + ?Sized>(&self, source: &S, _edits: &[TextEdit], cache: &'a mut impl LexerCache<MermaidLanguage>) -> LexOutput<MermaidLanguage> {
        let mut state: State<'_, S> = LexerState::new(source);
        let result = self.run(&mut state);
        if result.is_ok() {
            state.add_eof();
        }
        state.finish_with_cache(result, cache)
    }
}

impl<'config> MermaidLexer<'config> {
    /// Creates a new Mermaid lexer with the given configuration.
    pub fn new(config: &'config MermaidLanguage) -> Self {
        Self { config }
    }

    fn run<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> Result<(), oak_core::OakError> {
        // 上一个非 trivia token，用于只在 `graph` 之后识别方向关键字。
        let mut last_significant: Option<MermaidTokenType> = None;

        while state.not_at_end() {
            let safe_point = state.get_position();

            // 换行：CRLF 视为单个 Newline token。
            if state.starts_with("\r\n") {
                state.advance(2);
                state.add_token(MermaidTokenType::Newline, safe_point, state.get_position());
                last_significant = Some(MermaidTokenType::Newline);
                continue;
            }
            if let Some(ch) = state.peek() {
                if ch == '\n' || ch == '\r' {
                    state.advance(ch.len_utf8());
                    state.add_token(MermaidTokenType::Newline, safe_point, state.get_position());
                    last_significant = Some(MermaidTokenType::Newline);
                    continue;
                }
                if ch == ' ' || ch == '\t' {
                    state.take_while(|c| c == ' ' || c == '\t');
                    state.add_token(MermaidTokenType::Whitespace, safe_point, state.get_position());
                    continue;
                }
            }

            // 注释：`%%` 到本行结尾（不含换行）。
            if state.starts_with("%%") {
                state.advance(2);
                state.skip_until(b'\n');
                state.add_token(MermaidTokenType::Comment, safe_point, state.get_position());
                continue;
            }

            // 箭头。
            if let Some(arrow) = ARROWS.iter().find(|arrow| state.starts_with(arrow)) {
                state.advance(arrow.len());
                state.add_token(MermaidTokenType::Arrow, safe_point, state.get_position());
                last_significant = Some(MermaidTokenType::Arrow);
                continue;
            }

            // 标签：节点标签（含括号）或边标签（`|..|`）。
            if let Some(end) = self.label_end(state) {
                state.advance(end - safe_point);
                state.add_token(MermaidTokenType::Label, safe_point, state.get_position());
                last_significant = Some(MermaidTokenType::Label);
                continue;
            }

            // 标识符与关键字。
            if let Some(byte) = state.peek_byte() {
                if byte == b'_' || byte.is_ascii_alphabetic() {
                    state.advance_byte();
                    self.skip_identifier_tail(state);

                    let kind = {
                        let text = state.get_text_in((safe_point..state.get_position()).into());
                        if text.as_ref() == "graph" || text.as_ref() == "flowchart" {
                            MermaidTokenType::Graph
                        }
                        else if last_significant == Some(MermaidTokenType::Graph) && DIRECTIONS.contains(&text.as_ref()) {
                            MermaidTokenType::Direction
                        }
                        else {
                            MermaidTokenType::Id
                        }
                    };
                    state.add_token(kind, safe_point, state.get_position());
                    last_significant = Some(kind);
                    continue;
                }
            }

            // 真正无法识别的字符。
            if let Some(ch) = state.peek() {
                state.advance(ch.len_utf8());
                state.add_token(MermaidTokenType::Error, safe_point, state.get_position());
                last_significant = Some(MermaidTokenType::Error);
            }

            state.advance_if_dead_lock(safe_point);
        }

        Ok(())
    }

    /// 跳过标识符的后续字符。
    ///
    /// 允许 `[A-Za-z0-9_.-]`，但遇到 `-` 时需要确认它不是箭头前缀
    /// （例如 `A-->B` 中的 `-`），否则会把 `-->` 吞进标识符。
    fn skip_identifier_tail<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) {
        while let Some(byte) = state.peek_byte() {
            if byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'.' {
                state.advance_byte();
            }
            else if byte == b'-' {
                match state.peek_next() {
                    Some(next) if next.is_ascii_alphanumeric() || next == '_' => {
                        state.advance_byte();
                    }
                    _ => break,
                }
            }
            else {
                break;
            }
        }
    }

    /// 返回标签 token 的结束位置（不含）。
    ///
    /// 支持 `[..]`、`(..)`、`{..}`、`((..))`、`[[..]]`、`([..])`、`[(..)]`
    /// 与边标签 `|..|`。先匹配更长的开括号，匹配不到闭括号时再退回较短的
    /// 形式，避免未闭合的括号吞掉整份源码。
    fn label_end<'s, S: Source + ?Sized>(&self, state: &mut State<'s, S>) -> Option<usize> {
        let start = state.get_position();
        let rest = state.rest();
        let candidates: &[(&str, &str)] = match rest.chars().next()? {
            '[' => &[("[[", "]]"), ("[(", ")]"), ("[", "]")],
            '(' => &[("((", "))"), ("([", "])"), ("(", ")")],
            '{' => &[("{", "}")],
            '|' => &[("|", "|")],
            _ => return None,
        };

        for (open, close) in candidates {
            let Some(body) = rest.strip_prefix(open)
            else {
                continue;
            };
            if let Some(index) = body.find(close) {
                return Some(start + open.len() + index + close.len());
            }
        }

        None
    }
}
