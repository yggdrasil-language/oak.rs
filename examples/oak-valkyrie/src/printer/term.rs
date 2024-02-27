//! Term 表达式打印（增量覆盖）。

use crate::ast::{StringLiteral, TermBinaryNode, TermExpression};

use super::{
    common::{print_name_path, print_operator},
    error::PrintError,
};

pub(crate) fn print_term(expression: &TermExpression) -> Result<String, PrintError> {
    match expression {
        TermExpression::Binary(node) => print_binary(node),
        TermExpression::IntegerLiteral { value, .. } => Ok(value.clone()),
        TermExpression::FloatLiteral { value, .. } => Ok(value.clone()),
        TermExpression::Bool { value, .. } => Ok(if *value { "true".to_string() } else { "false".to_string() }),
        TermExpression::StringLiteral(literal) => Ok(print_string_literal(literal)),
        TermExpression::NamePath(path) => Ok(print_name_path(path)),
        other => Err(PrintError::Unsupported { context: format!("term expression `{other:?}`") }),
    }
}

fn print_binary(node: &TermBinaryNode) -> Result<String, PrintError> {
    let operator = print_operator(node.operator).ok_or_else(|| PrintError::Unsupported { context: format!("binary operator `{:?}`", node.operator) })?;
    let lhs = print_term(&node.lhs)?;
    let rhs = print_term(&node.rhs)?;
    Ok(format!("{lhs}{operator}{rhs}"))
}

fn print_string_literal(literal: &StringLiteral) -> String {
    let mut out = String::new();
    out.push('"');
    for segment in &literal.segments {
        match segment {
            crate::ast::StringSegment::Text(text) => out.push_str(&text.content),
            crate::ast::StringSegment::Interpolation(_) => out.push_str("…"),
        }
    }
    out.push('"');
    out
}
