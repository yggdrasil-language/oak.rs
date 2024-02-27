//! AST **pretty printer**（已解析模型 → 文本；不保留注释与空白）。
//!
//! 与 [`crate::formatter`]（CST token-gap 源码格式化）严格分离。
//!
//! 当前为增量 scaffold：仅覆盖 `namespace` / `micro` 等小子集，其余节点返回 [`PrintError::Unsupported`]。

mod block;
mod common;
mod error;
mod items;
mod options;
mod stmt;
mod term;

pub use error::PrintError;
pub use options::{PrintOptions, PrintStyle};

use oak_core::{Builder, ParseSession, SourceText};

use crate::{ValkyrieBuilder, ValkyrieLanguage, ast::ValkyrieRoot};

use items::print_item;

/// 解析 Valkyrie 源码为 AST 根节点。
pub fn parse_source(source: &str) -> Result<ValkyrieRoot, PrintError> {
    if source.is_empty() {
        return Ok(ValkyrieRoot { items: Vec::new() });
    }

    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let text = SourceText::new(source.to_string());
    let mut cache = ParseSession::default();
    let built = builder.build(&text, &[], &mut cache);

    if let Err(error) = &built.result {
        return Err(PrintError::Parse(format!("{error:?}")));
    }
    if !built.diagnostics.is_empty() {
        return Err(PrintError::Parse(format!("diagnostics: {:?}", built.diagnostics)));
    }

    built.result.ok().ok_or_else(|| PrintError::Parse("build returned no root".into()))
}

/// 将 AST 根节点写出为文本。
pub fn print_root(root: &ValkyrieRoot, style: PrintStyle, options: &PrintOptions) -> Result<String, PrintError> {
    let mut out = String::new();
    for (index, item) in root.items.iter().enumerate() {
        if index > 0 {
            match style {
                PrintStyle::Compact => out.push(' '),
                PrintStyle::Indented => out.push('\n'),
            }
        }
        out.push_str(&print_item(item, style, options)?);
    }
    Ok(out)
}

/// 解析并打印 Valkyrie 源码（AST print 路径，非 formatter）。
pub fn print_source(source: &str, style: PrintStyle, options: &PrintOptions) -> Result<String, PrintError> {
    let root = parse_source(source)?;
    let mut out = print_root(&root, style, options)?;
    if source.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}
