//! CST-oriented `source` formatter (green-tree spans + trivia gaps).
//!
//! Uses `TypeScriptParser` for the concrete tree, preserves verbatim text between
//! top-level statement spans, and formats supported statements via transitional AST print.
//! This is the path toward real `oak-formatter` / `oak-pretty-print` integration — not a
//! finished CST formatter yet.

use oak_core::{ParseSession, Parser, RedNode, RedTree, SourceText};

use crate::{
    language::TypeScriptLanguage,
    parser::{TypeScriptParser, element_type::TypeScriptElementType},
};

use super::{cst_options::CstFormatOptions, red_tree::TypeScriptRedTreeFormatter};

/// Format a parsed `SourceFile` red node with companion source text.
pub(crate) fn format_source_file(source: &str, file: &RedNode<'_, TypeScriptLanguage>, options: &CstFormatOptions) -> Result<String, String> {
    if file.element_type() != TypeScriptElementType::SourceFile {
        return Err(format!("expected SourceFile root, got {:?}", file.element_type()));
    }

    let file_span = file.span();
    let mut out = String::new();
    let mut cursor = file_span.start;

    for child in file.children() {
        let child_span = child.span();
        if child_span.start < cursor {
            return Err("overlapping CST spans".into());
        }
        if child_span.start > cursor {
            out.push_str(slice_source(source, cursor, child_span.start));
        }

        match child {
            RedTree::Leaf(_) => {
                out.push_str(slice_source(source, child_span.start, child_span.end));
            }
            RedTree::Node(stmt) => {
                let kind = stmt.element_type();
                if !is_supported_top_level(kind) {
                    return Err(format!("unsupported top-level CST node: {kind:?}"));
                }
                out.push_str(&TypeScriptRedTreeFormatter::new().format_statement(source, &stmt, options)?);
            }
        }
        cursor = child_span.end;
    }

    if cursor < file_span.end {
        out.push_str(slice_source(source, cursor, file_span.end));
    }
    else if cursor < source.len() {
        out.push_str(slice_source(source, cursor, source.len()));
    }

    Ok(options.finalize_output(source, out))
}

/// Format TypeScript/JavaScript source with trivia between top-level statements preserved.
pub(crate) fn format_source(source: &str, options: &CstFormatOptions) -> Result<String, String> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = TypeScriptLanguage::default();
    let parser = TypeScriptParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&text, &[], &mut session);

    if let Err(err) = &parsed.result {
        return Err(format!("oak parse failed: {err:?}"));
    }
    if !parsed.diagnostics.is_empty() {
        return Err(format!("oak diagnostics: {:?}", parsed.diagnostics));
    }

    let root_green = parsed.result.ok().ok_or_else(|| "oak parse returned no root".to_string())?;
    let file = RedNode::new(root_green, 0);
    format_source_file(source, &file, options)
}

fn slice_source(source: &str, start: usize, end: usize) -> &str {
    source.get(start..end).unwrap_or("")
}

fn is_supported_top_level(kind: TypeScriptElementType) -> bool {
    matches!(kind, TypeScriptElementType::VariableDeclaration | TypeScriptElementType::ImportDeclaration | TypeScriptElementType::ExportDeclaration | TypeScriptElementType::ExpressionStatement)
}
