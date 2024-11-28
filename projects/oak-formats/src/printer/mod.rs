//! AST **pretty printer** 注册表（平台契约见 [`crate::contract`]）。
//!
//! 与 [`crate::formatter`]（CST 源码正规格式化）是两套完全不同的概念，对齐 Oak 各语言 crate 的
//! `printer/` 与 `formatter/` 并列目录。
//!
//! - **Printer**：已解析 AST / 值模型 → 文本；不保留注释与空白。
//! - **Formatter**：源码 → CST token-gap → 文本；保留 trivia。
//!
//! 具体语言的 printer 由拥有该格式的 crate 通过 [`register_printer_provider`] 或 [`register_printer`] 注册；
//! 本 crate 只直接注册 VON 这类由 Oak 自己拥有的文本模型，不反向依赖任何格式域 crate。

use std::{
    any::Any,
    sync::{OnceLock, RwLock, RwLockReadGuard},
};

use crate::contract::{FormatError, FormatOptions, PrintStyle, Printer, PrinterProvider, PrinterRegistry};
use oak_von::{
    VonValue,
    printer::{PrintOptions as VonPrintOptions, PrintStyle as VonPrintStyle, print_value},
};

fn map_von_print_style(style: PrintStyle) -> VonPrintStyle {
    match style {
        PrintStyle::Compact => VonPrintStyle::Compact,
        PrintStyle::Indented => VonPrintStyle::Indented,
    }
}

fn map_von_print_options(style: PrintStyle, options: &FormatOptions) -> VonPrintOptions {
    VonPrintOptions { style: map_von_print_style(style), indent_width: options.indent_width, max_width: options.max_width }
}

fn print_von_value(value: &VonValue, style: PrintStyle, options: &FormatOptions) -> String {
    print_value(value, &map_von_print_options(style, options))
}

struct VonPrinter;

impl Printer for VonPrinter {
    fn language_id(&self) -> &str {
        "von"
    }

    fn print(&self, document: &dyn Any, style: PrintStyle, options: &FormatOptions) -> Result<String, FormatError> {
        let value = document.downcast_ref::<VonValue>().ok_or_else(|| FormatError::WrongDocument { expected: "VonValue".into() })?;
        Ok(print_von_value(value, style, options))
    }
}

/// 平台 printer 注册表（启动时由各格式域 crate 补全）。
///
/// 读取方应通过 [`print_document`] 及其平台包装函数间接访问。
pub fn printer_registry() -> &'static RwLock<PrinterRegistry> {
    static REGISTRY: OnceLock<RwLock<PrinterRegistry>> = OnceLock::new();
    REGISTRY.get_or_init(|| {
        let mut registry = PrinterRegistry::new();
        registry.register(&["von"], Box::new(VonPrinter));
        RwLock::new(registry)
    })
}

/// 注册格式域 printer provider（线程安全，可重复调用；先注册者优先）。
pub fn register_printer_provider(provider: &dyn PrinterProvider) {
    printer_registry().write().unwrap_or_else(|error| error.into_inner()).register_provider(provider);
}

/// 直接注册一个 printer（供拥有格式模型的 crate 装配，不引入反向依赖）。
pub fn register_printer(language_ids: &[&str], printer: Box<dyn Printer>) {
    printer_registry().write().unwrap_or_else(|error| error.into_inner()).register(language_ids, printer);
}

fn printer_registry_read() -> RwLockReadGuard<'static, PrinterRegistry> {
    printer_registry().read().unwrap_or_else(|error| error.into_inner())
}

/// 按语言 id 将已解析文档写出为文本。
pub fn print_document(language_id: &str, document: &dyn Any, style: PrintStyle, options: &FormatOptions) -> Result<String, FormatError> {
    printer_registry_read().print(language_id, document, style, options)
}

/// 打印 `VonValue`（AST pretty print，非 CST formatter）。
pub fn print_von(value: &VonValue, style: PrintStyle, options: &FormatOptions) -> Result<String, FormatError> {
    print_document("von", value, style, options)
}

/// 通过 serde 将 `value` 序列化为紧凑 VON 文本（AST print 路径）。
#[cfg(feature = "serde")]
pub fn to_string<T>(value: &T) -> Result<String, oak_core::OakError>
where
    T: serde::Serialize,
{
    oak_von::to_string(value)
}

/// 经 `oak-von::printer` 将 `value` 写出为缩进 VON 文本。
#[cfg(feature = "serde")]
pub fn to_string_indented<T>(value: &T) -> Result<String, oak_core::OakError>
where
    T: serde::Serialize,
{
    oak_von::to_string_indented(value, FormatOptions::default().indent_width)
}
