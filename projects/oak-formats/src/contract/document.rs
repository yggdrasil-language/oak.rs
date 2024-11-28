//! Wadler 风格 pretty-print 文档代数与布局引擎。

use std::ops::Range;

use super::{FormatOptions, source_map::SourceMap};

/// Pretty-print 文档。
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Document {
    /// 空文档。
    Nil,
    /// 不可拆分文本。
    Text(String),
    /// 原文 trivia（原样输出，不参与 group 宽度计算）。
    Trivia {
        /// 原文切片。
        text: String,
        /// 原文字节范围（用于 SourceMap）。
        orig_span: Option<Range<usize>>,
    },
    /// 顺序拼接。
    Concat(Box<Document>, Box<Document>),
    /// 子文档在换行后增加 `n` 级缩进（级宽由 [`FormatOptions::indent_width`] 决定）。
    Nest(usize, Box<Document>),
    /// 软断点：压平时为 `flat`，折行时为换行。
    SoftBreak {
        /// 压平后的替代文本。
        flat: String,
    },
    /// 硬换行（不可压平）。
    HardLine,
    /// 若整组能压平放入剩余宽度则压平，否则按折行布局。
    Group(Box<Document>),
    /// 多行填充：放不下则每项独占一行（Biome `fill`）。
    Fill {
        /// 项间分隔（常为 `,` + softline）。
        separator: Box<Document>,
        /// 子项。
        items: Vec<Document>,
    },
}

impl Document {
    /// 空文档。
    pub fn nil() -> Self {
        Self::Nil
    }

    /// 原文 trivia（不参与布局宽度）。
    pub fn trivia(text: impl Into<String>) -> Self {
        let text = text.into();
        if text.is_empty() { Self::Nil } else { Self::Trivia { text, orig_span: None } }
    }

    /// 带原文范围的 trivia。
    pub fn trivia_span(text: impl Into<String>, orig_span: Range<usize>) -> Self {
        let text = text.into();
        if text.is_empty() { Self::Nil } else { Self::Trivia { text, orig_span: Some(orig_span) } }
    }

    /// 用 `separator` 连接若干文档；放不下则每项折行（`fill`）。
    pub fn fill(separator: Self, items: impl IntoIterator<Item = Self>) -> Self {
        let items: Vec<_> = items.into_iter().filter(|d| !matches!(d, Self::Nil)).collect();
        if items.is_empty() {
            Self::Nil
        }
        else if items.len() == 1 {
            items.into_iter().next().unwrap()
        }
        else {
            Self::Fill { separator: Box::new(separator), items }
        }
    }

    /// 拼接 trivia 与语法文档（保留顺序）。
    pub fn join_with_trivia(parts: impl IntoIterator<Item = Self>) -> Self {
        let mut iter = parts.into_iter();
        let Some(first) = iter.next()
        else {
            return Self::Nil;
        };
        let mut out = first;
        for part in iter {
            out = out.append(part);
        }
        out
    }

    /// 文本节点。
    pub fn text(s: impl Into<String>) -> Self {
        let s = s.into();
        if s.is_empty() { Self::Nil } else { Self::Text(s) }
    }

    /// 单个空格。
    pub fn space() -> Self {
        Self::text(" ")
    }

    /// 经典 soft line：压平为空格，折行时换行。
    pub fn line() -> Self {
        Self::SoftBreak { flat: " ".into() }
    }

    /// softline：压平为空，折行时换行。
    pub fn softline() -> Self {
        Self::SoftBreak { flat: String::new() }
    }

    /// 硬换行。
    pub fn hardline() -> Self {
        Self::HardLine
    }

    /// 拼接。
    pub fn append(self, other: Self) -> Self {
        match (self, other) {
            (Self::Nil, document) | (document, Self::Nil) => document,
            (left, right) => Self::Concat(Box::new(left), Box::new(right)),
        }
    }

    /// 增加 `levels` 级缩进。
    pub fn nest(self, levels: usize) -> Self {
        if levels == 0 || matches!(self, Self::Nil) { self } else { Self::Nest(levels, Box::new(self)) }
    }

    /// 成组：能放下则压平，否则折行。
    pub fn group(self) -> Self {
        match self {
            Self::Nil => Self::Nil,
            other => Self::Group(Box::new(other)),
        }
    }

    /// 用 `separator` 连接若干文档。
    pub fn join(separator: Self, documents: impl IntoIterator<Item = Self>) -> Self {
        let mut iter = documents.into_iter();
        let Some(first) = iter.next()
        else {
            return Self::Nil;
        };
        let mut out = first;
        for document in iter {
            out = out.append(separator.clone()).append(document);
        }
        out
    }

    /// `{` body `}`，body nest(1)，两侧 softline，整体 group。
    pub fn braces(body: Self) -> Self {
        Self::enclose_with(Self::text("{"), body, Self::text("}"))
    }

    /// `[` body `]`。
    pub fn brackets(body: Self) -> Self {
        Self::enclose_with(Self::text("["), body, Self::text("]"))
    }

    /// 自定义开闭包裹。
    pub fn enclose_with(open: Self, body: Self, close: Self) -> Self {
        open.append(Self::softline().append(body).nest(1)).append(Self::softline()).append(close).group()
    }

    /// 强制压平渲染（硬换行仍保留）。
    pub fn render_compact(&self, options: &FormatOptions) -> String {
        render_document(self, options, true)
    }

    /// 按行宽 pretty 渲染。
    pub fn render(&self, options: &FormatOptions) -> String {
        self.render_with_map(options, false).0
    }

    /// 渲染并构建 SourceMap（trivia 段按 `orig_span` 映射）。
    pub fn render_with_map(&self, options: &FormatOptions, force_flat: bool) -> (String, SourceMap) {
        let width = if force_flat { usize::MAX } else { options.max_width };
        let mut out = String::new();
        let mut map = SourceMap::new();
        let mut stack: Vec<(usize, Mode, &Document)> = vec![(0, if force_flat { Mode::Flat } else { Mode::Break }, self)];
        let mut col = 0usize;

        while let Some((indent, mode, current)) = stack.pop() {
            match current {
                Document::Nil => {}
                Document::Text(s) => {
                    let start = out.len();
                    out.push_str(s);
                    col = col.saturating_add(s.chars().count());
                    map.push(start..start, start..out.len());
                }
                Document::Trivia { text, orig_span } => {
                    let start = out.len();
                    out.push_str(text);
                    col = col.saturating_add(text.chars().count());
                    if let Some(span) = orig_span {
                        map.push(span.clone(), start..out.len());
                    }
                }
                Document::Concat(a, b) => {
                    stack.push((indent, mode, b));
                    stack.push((indent, mode, a));
                }
                Document::Nest(n, inner) => {
                    stack.push((indent.saturating_add(*n), mode, inner));
                }
                Document::SoftBreak { flat } => match mode {
                    Mode::Flat => {
                        out.push_str(flat);
                        col = col.saturating_add(flat.chars().count());
                    }
                    Mode::Break => {
                        out.push('\n');
                        let spaces = indent * options.indent_width;
                        for _ in 0..spaces {
                            out.push(' ');
                        }
                        col = spaces;
                    }
                },
                Document::HardLine => {
                    out.push('\n');
                    let spaces = indent * options.indent_width;
                    for _ in 0..spaces {
                        out.push(' ');
                    }
                    col = spaces;
                }
                Document::Group(inner) => {
                    let next_mode = if force_flat {
                        Mode::Flat
                    }
                    else if fits(width.saturating_sub(col), indent, Mode::Flat, inner) {
                        Mode::Flat
                    }
                    else {
                        Mode::Break
                    };
                    stack.push((indent, next_mode, inner));
                }
                Document::Fill { separator, items } => {
                    if force_flat || fits_fill(width.saturating_sub(col), indent, separator, items) {
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                stack.push((indent, Mode::Flat, separator.as_ref()));
                            }
                            stack.push((indent, Mode::Flat, item));
                        }
                    }
                    else {
                        for (i, item) in items.iter().enumerate() {
                            if i > 0 {
                                stack.push((indent, Mode::Break, separator.as_ref()));
                            }
                            stack.push((indent, Mode::Break, item));
                        }
                    }
                }
            }
        }

        (out, map)
    }
}

impl From<&str> for Document {
    fn from(value: &str) -> Self {
        Self::text(value)
    }
}

impl From<String> for Document {
    fn from(value: String) -> Self {
        Self::text(value)
    }
}

#[derive(Clone, Copy)]
enum Mode {
    Flat,
    Break,
}

fn render_document(document: &Document, options: &FormatOptions, force_flat: bool) -> String {
    document.render_with_map(options, force_flat).0
}

fn fits_fill(remain: usize, indent: usize, separator: &Document, items: &[Document]) -> bool {
    let mut remain = remain as isize;
    for (i, item) in items.iter().enumerate() {
        if i > 0 && !fits(remain.max(0) as usize, indent, Mode::Flat, separator) {
            return false;
        }
        if !fits(remain.max(0) as usize, indent, Mode::Flat, item) {
            return false;
        }
        remain -= measure_flat(item) as isize;
        if i > 0 {
            remain -= measure_flat(separator) as isize;
        }
    }
    remain >= 0
}

fn measure_flat(document: &Document) -> usize {
    match document {
        Document::Nil => 0,
        Document::Text(s) | Document::Trivia { text: s, .. } => s.chars().count(),
        Document::Concat(a, b) => measure_flat(a) + measure_flat(b),
        Document::Nest(_, inner) => measure_flat(inner),
        Document::SoftBreak { flat } => flat.chars().count(),
        Document::HardLine => 0,
        Document::Group(inner) => measure_flat(inner),
        Document::Fill { separator, items } => {
            let sep = measure_flat(separator);
            items.iter().map(measure_flat).sum::<usize>() + sep.saturating_mul(items.len().saturating_sub(1))
        }
    }
}

fn fits(remain: usize, indent: usize, mode: Mode, document: &Document) -> bool {
    let mut remain = remain as isize;
    let mut stack: Vec<(usize, Mode, &Document)> = vec![(indent, mode, document)];

    while let Some((indent, mode, current)) = stack.pop() {
        if remain < 0 {
            return false;
        }
        match current {
            Document::Nil => {}
            Document::Text(s) => {
                remain -= s.chars().count() as isize;
            }
            Document::Trivia { text, .. } => {
                remain -= text.chars().count() as isize;
            }
            Document::Concat(a, b) => {
                stack.push((indent, mode, b));
                stack.push((indent, mode, a));
            }
            Document::Nest(n, inner) => {
                stack.push((indent + *n, mode, inner));
            }
            Document::SoftBreak { flat } => match mode {
                Mode::Flat => {
                    remain -= flat.chars().count() as isize;
                }
                Mode::Break => {
                    return true;
                }
            },
            Document::HardLine => {
                return true;
            }
            Document::Group(inner) => {
                stack.push((indent, Mode::Flat, inner));
            }
            Document::Fill { separator, items } => {
                for (i, item) in items.iter().enumerate() {
                    if i > 0 {
                        stack.push((indent, Mode::Flat, separator.as_ref()));
                    }
                    stack.push((indent, Mode::Flat, item));
                }
            }
        }
    }

    remain >= 0
}
