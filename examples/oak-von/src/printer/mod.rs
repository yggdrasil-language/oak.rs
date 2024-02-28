//! AST **pretty printer**（已解析模型 → `Document` → 文本；不保留注释与空白）。
//!
//! 与 [`crate::formatter`]（CST token-gap 源码格式化）严格分离。

mod options;

pub use options::{PrintOptions, PrintStyle};

use oak_pretty_print::{AsDocument, Document};

use crate::{
    ast::VonValue as AstVonValue,
    language::value::{VonValue, to_ast},
};

/// 将 AST 转为 pretty-print 文档（布局决策在 `render_*` 阶段）。
pub fn to_document(value: &AstVonValue) -> Document<'_> {
    value.as_document(&())
}

/// 渲染 `Document` 为文本。
pub fn render_document(doc: &Document<'_>, options: &PrintOptions) -> String {
    doc.render_with_config(options.printer_config())
}

/// 将 AST 节点写出为文本。
pub fn print_ast(value: &AstVonValue, options: &PrintOptions) -> String {
    render_document(&to_document(value), options)
}

/// 将语言值模型写出为文本。
pub fn print_value(value: &VonValue, options: &PrintOptions) -> String {
    let ast = to_ast(value);
    print_ast(&ast, options)
}
