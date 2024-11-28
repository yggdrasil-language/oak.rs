//! 格式化前后字节偏移映射。

use std::ops::Range;

use super::syntax::ByteRange;

/// 单段映射：格式化输出中的一段对应原文中的一段。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceMapEntry {
    /// 原文范围。
    pub original: ByteRange,
    /// 格式化后范围。
    pub formatted: ByteRange,
}

/// 格式化前后偏移映射（分段线性）。
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceMap {
    entries: Vec<SourceMapEntry>,
    formatted_len: usize,
    original_len: usize,
}

impl SourceMap {
    /// 空映射。
    pub fn new() -> Self {
        Self::default()
    }

    /// 记录一段一一对应映射。
    pub fn push(&mut self, original: Range<usize>, formatted: Range<usize>) {
        self.original_len = self.original_len.max(original.end);
        self.formatted_len = self.formatted_len.max(formatted.end);
        self.entries.push(SourceMapEntry { original: ByteRange::new(original), formatted: ByteRange::new(formatted) });
    }

    /// 格式化后文本长度。
    pub fn formatted_len(&self) -> usize {
        self.formatted_len
    }

    /// 原文长度。
    pub fn original_len(&self) -> usize {
        self.original_len
    }

    /// 原文偏移 → 格式化偏移（最近映射点）。
    pub fn orig_to_formatted(&self, offset: usize) -> usize {
        for entry in &self.entries {
            if entry.original.contains(offset) {
                let delta = offset.saturating_sub(entry.original.start);
                return entry.formatted.start.saturating_add(delta);
            }
        }
        if offset >= self.original_len {
            return self.formatted_len;
        }
        offset
    }

    /// 格式化偏移 → 原文偏移。
    pub fn formatted_to_orig(&self, offset: usize) -> usize {
        for entry in &self.entries {
            if entry.formatted.contains(offset) {
                let delta = offset.saturating_sub(entry.formatted.start);
                return entry.original.start.saturating_add(delta);
            }
        }
        if offset >= self.formatted_len {
            return self.original_len;
        }
        offset
    }

    /// 所有映射条目（只读）。
    pub fn entries(&self) -> &[SourceMapEntry] {
        &self.entries
    }
}

/// 格式化结果（文本 + 偏移映射）。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FormattedOutput {
    /// 格式化后文本。
    pub text: String,
    /// 偏移映射。
    pub map: SourceMap,
}

impl FormattedOutput {
    /// 仅文本（无映射）。
    pub fn text_only(text: String) -> Self {
        let len = text.len();
        let mut map = SourceMap::new();
        map.push(0..len, 0..len);
        Self { text, map }
    }
}
