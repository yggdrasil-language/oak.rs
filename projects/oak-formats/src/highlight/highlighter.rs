//! 高亮器契约（对齐 JetBrains：同语言可有 Lexical / Semantic 多种 Highlighter）。

use std::{any::Any, collections::HashMap};

use super::{HighlightKind, HighlightSpan, merge_spans};

/// 高亮 pass 种类。
///
/// 同一语言可注册多个不同 kind 的高亮器：词法快速、语义高质量。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HighlighterKind {
    /// 基于词法的快速着色，只需源文本。
    Lexical,
    /// 基于语义分析的高质量着色，需要分析上下文。
    Semantic,
}

/// 前端注入的分析句柄；平台不定义 HIR / 符号表类型。
pub trait AnalysisContext: Send + Sync {
    /// 向下转型到具体分析模型。
    fn as_any(&self) -> &dyn Any;
}

/// 一次高亮请求。
pub struct HighlightRequest<'a> {
    /// 源文本。
    pub source: &'a str,
    /// 可选语义上下文；Lexical pass 可忽略；Semantic 在 `None` 时返回空 spans。
    pub analysis: Option<&'a dyn AnalysisContext>,
}

impl<'a> HighlightRequest<'a> {
    /// 仅源文本（文档 SSG / 片段场景）。
    pub fn source_only(source: &'a str) -> Self {
        Self { source, analysis: None }
    }

    /// 带分析上下文。
    pub fn with_analysis(source: &'a str, analysis: &'a dyn AnalysisContext) -> Self {
        Self { source, analysis: Some(analysis) }
    }
}

/// 语言无关的着色器入口。
///
/// 具体语言由各自前端 crate 为每个 pass 实现此 trait，
/// 同一 `language_id` 可注册多个 kind。
pub trait Highlighter: Send + Sync {
    /// 不透明语言标识（由插件约定，本仓不解释语义）。
    fn language_id(&self) -> &str;

    /// 本实例的 pass 种类。
    fn kind(&self) -> HighlighterKind;

    /// 对请求着色，返回按起点排序的片段。
    fn highlight(&self, request: &HighlightRequest<'_>) -> Vec<HighlightSpan>;
}

/// Token / node kind 到 `HighlightKind` 的分类函数。
pub type KindClassifier = dyn Fn(i32) -> HighlightKind + Send + Sync;

/// 基于映射表的通用语法高亮映射工具（对齐 C# `SyntaxHighlighter`）。
///
/// 不是「语言唯一 Highlighter」，语言插件用来把 token/node 映射为 `HighlightKind`。
#[derive(Default)]
pub struct SyntaxHighlighter {
    token_kind_map: HashMap<i32, HighlightKind>,
    node_kind_map: HashMap<i32, HighlightKind>,
    token_classifier: Option<Box<KindClassifier>>,
    node_classifier: Option<Box<KindClassifier>>,
}

impl SyntaxHighlighter {
    /// 空映射。
    pub fn new() -> Self {
        Self::default()
    }

    /// 使用回调分类器构造。
    pub fn with_classifiers(token: Box<KindClassifier>, node: Box<KindClassifier>) -> Self {
        Self { token_kind_map: HashMap::new(), node_kind_map: HashMap::new(), token_classifier: Some(token), node_classifier: Some(node) }
    }

    /// 注册 token kind 映射。
    pub fn map_token_kind(&mut self, token_type: i32, highlight_kind: HighlightKind) {
        self.token_kind_map.insert(token_type, highlight_kind);
    }

    /// 注册 node kind 映射。
    pub fn map_node_kind(&mut self, node_kind: i32, highlight_kind: HighlightKind) {
        self.node_kind_map.insert(node_kind, highlight_kind);
    }

    /// 分类 token。
    pub fn classify_token(&self, token_type: i32) -> HighlightKind {
        if let Some(kind) = self.token_kind_map.get(&token_type) {
            return *kind;
        }
        self.token_classifier.as_ref().map(|f| f(token_type)).unwrap_or(HighlightKind::None)
    }

    /// 分类 AST node。
    pub fn classify_node(&self, node_kind: i32) -> HighlightKind {
        if let Some(kind) = self.node_kind_map.get(&node_kind) {
            return *kind;
        }
        self.node_classifier.as_ref().map(|f| f(node_kind)).unwrap_or(HighlightKind::None)
    }

    /// 对 token 序列着色。
    pub fn highlight_tokens(&self, tokens: &[(i32, std::ops::Range<usize>)]) -> Vec<HighlightSpan> {
        tokens
            .iter()
            .filter_map(|(kind, range)| {
                let highlight = self.classify_token(*kind);
                (highlight != HighlightKind::None).then(|| HighlightSpan::new(highlight, range.clone()))
            })
            .collect()
    }

    /// 对 node 序列着色。
    pub fn highlight_nodes(&self, nodes: &[(i32, std::ops::Range<usize>)]) -> Vec<HighlightSpan> {
        nodes
            .iter()
            .filter_map(|(kind, range)| {
                let highlight = self.classify_node(*kind);
                (highlight != HighlightKind::None).then(|| HighlightSpan::new(highlight, range.clone()))
            })
            .collect()
    }
}

/// 中性语义 kind 映射（符号 kind id → `HighlightKind`），对齐 C# `SemanticHighlighter`。
#[derive(Debug, Default, Clone)]
pub struct SemanticKindMap {
    map: HashMap<i32, HighlightKind>,
}

impl SemanticKindMap {
    /// 空映射。
    pub fn new() -> Self {
        Self::default()
    }

    /// 注册映射。
    pub fn map(&mut self, symbol_kind: i32, highlight_kind: HighlightKind) {
        self.map.insert(symbol_kind, highlight_kind);
    }

    /// 分类。
    pub fn classify(&self, symbol_kind: i32) -> HighlightKind {
        self.map.get(&symbol_kind).copied().unwrap_or(HighlightKind::None)
    }
}

/// 按语言 id 解析高亮器工厂（一语言可产出多个 kind）。
pub trait HighlighterProvider {
    /// 支持的语言 id 列表。
    fn language_ids(&self) -> &[&str];

    /// 该语言的全部高亮器实例（Lexical、Semantic 等）。
    fn highlighters(&self) -> Vec<Box<dyn Highlighter>>;

    /// 按 kind 取首个匹配实例。
    fn highlighter(&self, kind: HighlighterKind) -> Option<Box<dyn Highlighter>> {
        self.highlighters().into_iter().find(|h| h.kind() == kind)
    }
}

/// 语言 → 多 pass 高亮器注册表。
#[derive(Default)]
pub struct HighlighterRegistry {
    /// `(canonical language_id, kind)` → 高亮器（每对仅保留首个注册）。
    by_lang_kind: HashMap<(String, HighlighterKind), Box<dyn Highlighter>>,
    /// 别名 → 规范 language_id。
    aliases: HashMap<String, String>,
}

impl HighlighterRegistry {
    /// 空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 provider 注册该语言的全部高亮器与语言别名。
    pub fn register_provider(&mut self, provider: &dyn HighlighterProvider) {
        let ids = provider.language_ids();
        if ids.is_empty() {
            return;
        }
        let canonical = ids[0].to_ascii_lowercase();
        for id in ids {
            self.aliases.insert(id.to_ascii_lowercase(), canonical.clone());
        }
        for highlighter in provider.highlighters() {
            let key = (canonical.clone(), highlighter.kind());
            self.by_lang_kind.entry(key).or_insert(highlighter);
        }
    }

    /// 直接注册一个高亮器（及其语言别名列表）。
    pub fn register(&mut self, language_ids: &[&str], highlighter: Box<dyn Highlighter>) {
        if language_ids.is_empty() {
            return;
        }
        let canonical = language_ids[0].to_ascii_lowercase();
        for id in language_ids {
            self.aliases.insert(id.to_ascii_lowercase(), canonical.clone());
        }
        let key = (canonical, highlighter.kind());
        self.by_lang_kind.entry(key).or_insert(highlighter);
    }

    fn canonicalize(&self, language_id: &str) -> String {
        let lower = language_id.to_ascii_lowercase();
        self.aliases.get(&lower).cloned().unwrap_or(lower)
    }

    /// 精确获取某一语言某一 kind。
    pub fn get(&self, language_id: &str, kind: HighlighterKind) -> Option<&dyn Highlighter> {
        let lang = self.canonicalize(language_id);
        self.by_lang_kind.get(&(lang, kind)).map(|h| h.as_ref())
    }

    /// 按偏好选择：优先 `preference`，不可用时回退 Lexical。
    ///
    /// `Semantic` 在无 `analysis` 时也回退 Lexical（调用方传入 request 信息以决定）。
    pub fn resolve<'a>(&'a self, language_id: &str, preference: HighlighterKind, has_analysis: bool) -> Option<&'a dyn Highlighter> {
        let want = match preference {
            HighlighterKind::Semantic if !has_analysis => HighlighterKind::Lexical,
            other => other,
        };
        self.get(language_id, want).or_else(|| self.get(language_id, HighlighterKind::Lexical))
    }

    /// 仅词法着色。
    pub fn highlight_lexical(&self, language_id: &str, source: &str) -> Option<Vec<HighlightSpan>> {
        let request = HighlightRequest::source_only(source);
        self.get(language_id, HighlighterKind::Lexical).map(|h| h.highlight(&request))
    }

    /// 词法 + 语义 overlay：同类区间语义覆盖词法。
    pub fn highlight_merged(&self, language_id: &str, request: &HighlightRequest<'_>) -> Option<Vec<HighlightSpan>> {
        let lexical = self.get(language_id, HighlighterKind::Lexical).map(|h| h.highlight(request)).unwrap_or_default();
        let semantic = self.get(language_id, HighlighterKind::Semantic).map(|h| h.highlight(request)).unwrap_or_default();
        if lexical.is_empty() && semantic.is_empty() {
            return None;
        }
        Some(merge_spans(lexical, semantic))
    }
}
