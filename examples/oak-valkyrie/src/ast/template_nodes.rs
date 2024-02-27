use super::{Span, StatementNode};
use crate::lexer::token_type::ValkyrieTokenType;
use oak_core::Token;
use std::hash::{Hash, Hasher};

/// `<% ... %>` 指令内部 token 流（不含 `TemplateL` / `TemplateR`）。
pub type TemplateTokenStream = Vec<Token<ValkyrieTokenType>>;

/// TGrammar 模板节点。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TemplateNode {
    /// 单行 meta `<% ... %>`（内部为完整 token 流，非 `TermExpression`）。
    Fragment {
        /// 指令内部 token 流。
        tokens: TemplateTokenStream,
        /// 源码跨度。
        #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
        span: Span,
    },
    /// `<% if %>` 条件块。
    If(TemplateIf),
    /// `<% loop %>` 循环块。
    Loop(TemplateLoop),
    /// `<% match %>` 匹配块。
    Match(TemplateMatch),
}

/// `<% if %>` 块。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateIf {
    /// 分支列表（含 `else`）。
    pub arms: Vec<TemplateIfArm>,
    /// 源码跨度。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// `if` 分支。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateIfArm {
    /// `<% if %>` / `<% else if %>` 之后的 token 流；`else` 分支为空。
    pub header: TemplateTokenStream,
    /// 分支体。
    pub body: Vec<StatementNode>,
    /// 源码跨度。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// `<% loop %>` 块。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateLoop {
    /// `<% loop %>` 之后的 token 流（如 `i` `in` `items`）。
    pub header: TemplateTokenStream,
    /// 循环体。
    pub body: Vec<StatementNode>,
    /// 源码跨度。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// `<% match %>` 块。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateMatch {
    /// `<% match %>` 之后的 token 流（被匹配对象）。
    pub header: TemplateTokenStream,
    /// 分支列表。
    pub arms: Vec<TemplateMatchArm>,
    /// 源码跨度。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

/// `match` 分支（`case` 或 `else`）。
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TemplateMatchArm {
    /// `<% case %>` 之后的模式 token 流；`else` 分支为 `None`。
    pub pattern: Option<TemplateTokenStream>,
    /// 分支体。
    pub body: Vec<StatementNode>,
    /// 源码跨度。
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Span,
}

fn hash_span<H: Hasher>(span: &Span, state: &mut H) {
    span.start.hash(state);
    span.end.hash(state);
}

fn hash_template_tokens<H: Hasher>(tokens: &TemplateTokenStream, state: &mut H) {
    for token in tokens {
        token.kind.hash(state);
        hash_span(&token.span, state);
    }
}

impl Hash for TemplateNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            Self::Fragment { tokens, span } => {
                0u8.hash(state);
                hash_template_tokens(tokens, state);
                hash_span(span, state);
            }
            Self::If(node) => {
                1u8.hash(state);
                node.hash(state);
            }
            Self::Loop(node) => {
                2u8.hash(state);
                node.hash(state);
            }
            Self::Match(node) => {
                3u8.hash(state);
                node.hash(state);
            }
        }
    }
}

impl Hash for TemplateIf {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.arms.hash(state);
        hash_span(&self.span, state);
    }
}

impl Hash for TemplateIfArm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_template_tokens(&self.header, state);
        self.body.hash(state);
        hash_span(&self.span, state);
    }
}

impl Hash for TemplateLoop {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_template_tokens(&self.header, state);
        self.body.hash(state);
        hash_span(&self.span, state);
    }
}

impl Hash for TemplateMatch {
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_template_tokens(&self.header, state);
        self.arms.hash(state);
        hash_span(&self.span, state);
    }
}

impl Hash for TemplateMatchArm {
    fn hash<H: Hasher>(&self, state: &mut H) {
        if let Some(pattern) = &self.pattern {
            hash_template_tokens(pattern, state);
        }
        self.body.hash(state);
        hash_span(&self.span, state);
    }
}
