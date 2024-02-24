//! 语句打印。

use crate::ast::{Let, Pattern, Statement};

use super::error::PrintError;
use super::term::print_term;

pub(crate) fn print_statement(statement: &Statement) -> Result<String, PrintError> {
    match statement {
        Statement::Let(let_stmt) => print_let(let_stmt),
        Statement::ExprStmt(expr_stmt) => {
            let mut out = print_term(&expr_stmt.expr)?;
            if expr_stmt.semi {
                out.push(';');
            }
            Ok(out)
        }
    }
}

fn print_let(let_stmt: &Let) -> Result<String, PrintError> {
    let mut out = String::from("let ");
    if let_stmt.is_mutable {
        out.push_str("mut ");
    }
    out.push_str(&print_pattern(&let_stmt.pattern)?);
    out.push('=');
    out.push_str(&print_term(&let_stmt.expr)?);
    Ok(out)
}

fn print_pattern(pattern: &Pattern) -> Result<String, PrintError> {
    match pattern {
        Pattern::Variable(variable) => Ok(variable.name.name.clone()),
        other => Err(PrintError::Unsupported { context: format!("pattern `{other:?}`") }),
    }
}
