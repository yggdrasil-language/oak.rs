//! Printer 契约：已解析模型 → 文本（序列化 / 调试，非源码 fmt）。

use std::{any::Any, collections::HashMap};

use super::{FormatError, FormatOptions};

/// 输出风格。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrintStyle {
    /// 紧凑单行（或最小空白）。
    Compact,
    /// 缩进多行。
    Indented,
}

/// 语言无关的 printer。
///
/// 具体语言由各自前端 crate 实现；`document` 由插件约定具体类型（经 `Any` 下转）。
pub trait Printer: Send + Sync {
    /// 不透明文档格式标识（由插件约定，本仓不解释语义）。
    fn language_id(&self) -> &str;

    /// 将文档写出为文本。
    fn print(&self, document: &dyn Any, style: PrintStyle, options: &FormatOptions) -> Result<String, FormatError>;
}

/// Printer 工厂。
pub trait PrinterProvider {
    /// 支持的语言 id 列表（首个为规范 id）。
    fn language_ids(&self) -> &[&str];

    /// 该格式的 printer。
    fn printer(&self) -> Box<dyn Printer>;
}

/// 语言 → printer 注册表。
#[derive(Default)]
pub struct PrinterRegistry {
    by_lang: HashMap<String, Box<dyn Printer>>,
    aliases: HashMap<String, String>,
}

impl PrinterRegistry {
    /// 空注册表。
    pub fn new() -> Self {
        Self::default()
    }

    /// 从 provider 注册。
    pub fn register_provider(&mut self, provider: &dyn PrinterProvider) {
        let ids = provider.language_ids();
        if ids.is_empty() {
            return;
        }
        let canonical = ids[0].to_ascii_lowercase();
        for id in ids {
            self.aliases.insert(id.to_ascii_lowercase(), canonical.clone());
        }
        self.by_lang.entry(canonical).or_insert_with(|| provider.printer());
    }

    /// 直接注册。
    pub fn register(&mut self, language_ids: &[&str], printer: Box<dyn Printer>) {
        if language_ids.is_empty() {
            return;
        }
        let canonical = language_ids[0].to_ascii_lowercase();
        for id in language_ids {
            self.aliases.insert(id.to_ascii_lowercase(), canonical.clone());
        }
        self.by_lang.entry(canonical).or_insert(printer);
    }

    fn canonicalize(&self, language_id: &str) -> String {
        let lower = language_id.to_ascii_lowercase();
        self.aliases.get(&lower).cloned().unwrap_or(lower)
    }

    /// 获取打印机。
    pub fn get(&self, language_id: &str) -> Option<&dyn Printer> {
        let lang = self.canonicalize(language_id);
        self.by_lang.get(&lang).map(|p| p.as_ref())
    }

    /// 按语言 id 写出文档。
    pub fn print(&self, language_id: &str, document: &dyn Any, style: PrintStyle, options: &FormatOptions) -> Result<String, FormatError> {
        match self.get(language_id) {
            Some(printer) => printer.print(document, style, options),
            None => Err(FormatError::unsupported_language(language_id)),
        }
    }
}
