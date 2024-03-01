//! AST **pretty printer**（已解析模型 → `Document` → 文本；不保留注释与空白）。
//!
//! 与 [`crate::formatter`]（CST token-gap 源码格式化）严格分离。
//!
//! 当前为增量 scaffold：仅覆盖 `namespace` / `micro` 等小子集，其余节点返回 [`PrintError::Unsupported`]。

mod document;
mod error;
mod options;

pub use error::PrintError;
pub use options::{PrintOptions, PrintStyle};

pub use oak_pretty_print::Document;

use crate::{ValkyrieBuilder, ValkyrieLanguage, ast::ValkyrieRoot};

use document::root_document;
use oak_core::{Builder, ParseSession, SourceText};

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

/// 将 AST 根节点转为 pretty-print 文档。
pub fn to_document(root: &ValkyrieRoot) -> Result<Document<'static>, PrintError> {
    root_document(root)
}

/// 渲染 `Document` 为文本。
pub fn render_document(doc: &Document<'_>, options: &PrintOptions) -> String {
    doc.render_with_config(options.printer_config())
}

/// 将 AST 根节点写出为文本。
pub fn print_root(root: &ValkyrieRoot, options: &PrintOptions) -> Result<String, PrintError> {
    Ok(render_document(&to_document(root)?, options))
}

/// 解析并打印 Valkyrie 源码（AST print 路径，非 formatter）。
pub fn print_source(source: &str, options: &PrintOptions) -> Result<String, PrintError> {
    let root = parse_source(source)?;
    let mut out = print_root(&root, options)?;
    if source.ends_with('\n') && !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}
