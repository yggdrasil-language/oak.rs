//! 顶层 item 打印。

use crate::ast::{MicroDeclaration, NamespaceDeclaration, StatementNode};

use super::{
    block::print_block,
    common::print_identifier,
    error::PrintError,
    options::{PrintOptions, PrintStyle},
};

pub(crate) fn print_item(item: &StatementNode, style: PrintStyle, options: &PrintOptions) -> Result<String, PrintError> {
    match item {
        StatementNode::Namespace(namespace) => print_namespace(namespace.as_ref(), style, options),
        StatementNode::Micro(micro) => print_micro(micro.as_ref(), style, options),
        other => Err(PrintError::Unsupported { context: format!("top-level item `{other:?}`") }),
    }
}

fn print_namespace(namespace: &NamespaceDeclaration, style: PrintStyle, options: &PrintOptions) -> Result<String, PrintError> {
    let name = namespace.name.parts.iter().map(|part| part.name.as_str()).collect::<Vec<_>>().join("::");
    if namespace.items.is_empty() {
        return Ok(format!("namespace {name};"));
    }
    let mut out = format!("namespace {name} {{");
    for item in &namespace.items {
        match style {
            PrintStyle::Compact => out.push(' '),
            PrintStyle::Indented => {
                out.push('\n');
                out.push_str(&" ".repeat(options.indent_width));
            }
        }
        out.push_str(&print_item(item, style, options)?);
    }
    out.push('}');
    Ok(out)
}

fn print_micro(micro: &MicroDeclaration, style: PrintStyle, options: &PrintOptions) -> Result<String, PrintError> {
    let mut out = String::from("micro ");
    out.push_str(&print_identifier(&micro.name));
    out.push('(');
    let params = micro.params.iter().map(|param| print_identifier(&param.name)).collect::<Vec<_>>().join(",");
    out.push_str(&params);
    out.push(')');
    out.push(' ');
    out.push_str(&print_block(&micro.body, style, options)?);
    Ok(out)
}
