//! 块打印。

use crate::ast::Block;

use super::error::PrintError;
use super::options::{PrintOptions, PrintStyle};
use super::stmt::print_statement;

pub(crate) fn print_block(block: &Block, style: PrintStyle, options: &PrintOptions) -> Result<String, PrintError> {
    if block.statements.is_empty() {
        return Ok("{}".to_string());
    }

    let mut body = String::new();
    for (index, statement) in block.statements.iter().enumerate() {
        match style {
            PrintStyle::Compact => {
                if index > 0 {
                    body.push(' ');
                }
            }
            PrintStyle::Indented => {
                body.push('\n');
                body.push_str(&" ".repeat(options.indent_width));
            }
        }
        body.push_str(&print_statement(statement)?);
    }

    match style {
        PrintStyle::Compact => Ok(format!("{{{body}}}")),
        PrintStyle::Indented => Ok(format!("{{{body}\n}}")),
    }
}
