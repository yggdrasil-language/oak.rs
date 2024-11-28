//! 语言无关 CST（Concrete Syntax Tree）基础设施。

use std::ops::Range;

/// 字节范围（半开区间）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ByteRange {
    /// 起始字节（含）。
    pub start: usize,
    /// 结束字节（不含）。
    pub end: usize,
}

impl ByteRange {
    /// 从 `Range<usize>` 构造。
    pub fn new(range: Range<usize>) -> Self {
        Self { start: range.start, end: range.end }
    }

    /// 转为 `Range<usize>`。
    pub fn as_range(self) -> Range<usize> {
        self.start..self.end
    }

    /// 是否包含偏移。
    pub fn contains(&self, offset: usize) -> bool {
        self.start <= offset && offset < self.end
    }

    /// 是否与另一范围相交。
    pub fn overlaps(&self, other: &Self) -> bool {
        self.start < other.end && other.start < self.end
    }
}

impl From<Range<usize>> for ByteRange {
    fn from(range: Range<usize>) -> Self {
        Self::new(range)
    }
}

/// 通用 trivia 种类（语言 lexer 可映射到此处或自有枚举）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriviaKind {
    /// 空白（空格、制表、换行等）。
    Whitespace,
    /// 行注释。
    LineComment,
    /// 块注释。
    BlockComment,
    /// 模板指令等其它非语法文本。
    Directive,
}

/// 词法记号（含 trivia）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    /// 语言相关种类标签（不透明；0 = trivia 由 `trivia_kind` 解释）。
    pub kind: u16,
    /// trivia 种类；仅当 `kind == 0` 时有效。
    pub trivia_kind: Option<TriviaKind>,
    /// 源码切片。
    pub text: String,
    /// 字节范围。
    pub span: ByteRange,
}

impl SyntaxToken {
    /// 构造 trivia token。
    pub fn trivia(kind: TriviaKind, text: impl Into<String>, span: Range<usize>) -> Self {
        Self { kind: 0, trivia_kind: Some(kind), text: text.into(), span: ByteRange::new(span) }
    }

    /// 是否为 trivia。
    pub fn is_trivia(&self) -> bool {
        self.trivia_kind.is_some()
    }
}

/// CST 子元素：token 或嵌套节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxElement {
    /// 词法记号（含 trivia）。
    Token(SyntaxToken),
    /// 语法节点。
    Node(SyntaxNode),
}

/// 语法树节点。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxNode {
    /// 语言相关节点种类（不透明）。
    pub kind: u16,
    /// 子元素（可含 leading/trailing trivia）。
    pub children: Vec<SyntaxElement>,
    /// 节点覆盖的字节范围。
    pub span: ByteRange,
    /// 错误恢复占位。
    pub is_error: bool,
}

impl SyntaxNode {
    /// 新建语法节点。
    pub fn new(kind: u16, span: Range<usize>, children: Vec<SyntaxElement>) -> Self {
        Self { kind, children, span: ByteRange::new(span), is_error: false }
    }

    /// 错误恢复节点。
    pub fn error(kind: u16, span: Range<usize>, message: impl Into<String>) -> Self {
        let text = message.into();
        Self {
            kind,
            children: vec![SyntaxElement::Token(SyntaxToken { kind: u16::MAX, trivia_kind: None, text, span: ByteRange::new(span.clone()) })],
            span: ByteRange::new(span),
            is_error: true,
        }
    }

    /// 收集所有 trivia 文本（按源码顺序）。
    pub fn leading_trivia_text(&self) -> String {
        let mut out = String::new();
        for child in &self.children {
            if let SyntaxElement::Token(token) = child {
                if token.is_trivia() {
                    out.push_str(&token.text);
                }
                else {
                    break;
                }
            }
            else {
                break;
            }
        }
        out
    }
}

/// CST 根（语言插件可包装为具体类型）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxRoot {
    /// 顶层子元素。
    pub children: Vec<SyntaxElement>,
    /// 全文范围。
    pub span: ByteRange,
}

impl SyntaxRoot {
    /// 空根。
    pub fn empty() -> Self {
        Self { children: Vec::new(), span: ByteRange { start: 0, end: 0 } }
    }
}
