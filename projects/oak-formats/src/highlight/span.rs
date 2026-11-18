//! 高亮片段。

use std::ops::Range;

use super::HighlightKind;

/// 一段已分类源文本（对齐 C# `HighlightToken`）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HighlightSpan {
    /// 着色种类。
    pub kind: HighlightKind,
    /// 字节区间（UTF-8）。
    pub range: Range<usize>,
    /// 可选修饰符（如 `declaration` / `readonly`）。
    pub modifier: Option<String>,
}

impl HighlightSpan {
    /// 无修饰符的片段。
    pub fn new(kind: HighlightKind, range: Range<usize>) -> Self {
        Self { kind, range, modifier: None }
    }

    /// 带修饰符的片段。
    pub fn with_modifier(kind: HighlightKind, range: Range<usize>, modifier: impl Into<String>) -> Self {
        Self { kind, range, modifier: Some(modifier.into()) }
    }
}

/// 合并词法与语义片段：相同起点区间由语义覆盖词法。
///
/// 典型用途：标识符从 `Identifier` 升级为 `TypeIdentifier` / `FunctionIdentifier`。
pub fn merge_spans(lexical: Vec<HighlightSpan>, semantic: Vec<HighlightSpan>) -> Vec<HighlightSpan> {
    if semantic.is_empty() {
        return sort_spans(lexical);
    }
    if lexical.is_empty() {
        return sort_spans(semantic);
    }

    let mut by_start: std::collections::BTreeMap<usize, HighlightSpan> = std::collections::BTreeMap::new();
    for span in lexical {
        by_start.insert(span.range.start, span);
    }
    for span in semantic {
        by_start.insert(span.range.start, span);
    }
    by_start.into_values().collect()
}

fn sort_spans(mut spans: Vec<HighlightSpan>) -> Vec<HighlightSpan> {
    spans.sort_by_key(|s| s.range.start);
    spans
}
