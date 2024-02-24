//! AST **pretty printer**（已解析模型 → 文本；不保留注释与空白）。
//!
//! 与 [`crate::formatter`]（CST token-gap 源码格式化）严格分离。

mod indented;
mod options;

pub use options::{PrintOptions, PrintStyle};

use oak_core::source::{SourceBuffer, ToSource};

use crate::ast::VonValue as AstVonValue;
use crate::language::value::{VonValue, to_ast};

/// 将 AST 节点写出为文本。
pub fn print_ast(value: &AstVonValue, style: PrintStyle, options: &PrintOptions) -> String {
    match style {
        PrintStyle::Compact => {
            let mut buffer = SourceBuffer::new();
            value.to_source(&mut buffer);
            buffer.to_string()
        }
        PrintStyle::Indented => indented::print_indented(value, 0, options),
    }
}

/// 将语言值模型写出为文本。
pub fn print_value(value: &VonValue, style: PrintStyle, options: &PrintOptions) -> String {
    print_ast(&to_ast(value), style, options)
}
