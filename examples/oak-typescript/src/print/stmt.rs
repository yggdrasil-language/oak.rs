//! Oak TypeScript statement → canonical text.

use crate::ast::{
    BlockStatement, ExportDeclaration, FunctionDeclaration, ImportDeclaration, ImportSpecifier,
    Statement, VariableDeclaration,
};

use super::expr::{print_expression, print_string};

/// Print one top-level or nested statement.
pub fn print_statement(stmt: &Statement) -> Option<String> {
    match stmt {
        Statement::VariableDeclaration(v) => print_variable(v),
        Statement::ExpressionStatement(es) => print_expression(&es.expression),
        Statement::ReturnStatement(r) => {
            let arg = match &r.argument {
                Some(e) => Some(print_expression(e)?),
                None => None,
            };
            Some(match arg {
                Some(e) => format!("return {e}"),
                None => "return".into(),
            })
        }
        Statement::ImportDeclaration(d) => print_import(d),
        Statement::ExportDeclaration(d) => print_export(d),
        Statement::FunctionDeclaration(d) => print_function(d),
        Statement::BlockStatement(b) => print_block(b),
        Statement::IfStatement(s) => {
            let test = print_expression(&s.test)?;
            let cons = print_statement(&s.consequent)?;
            let alt = match &s.alternate {
                Some(a) => format!(" else {}", print_statement(a)?),
                None => String::new(),
            };
            Some(format!("if ({test}) {cons}{alt}"))
        }
        Statement::WhileStatement(s) => {
            let test = print_expression(&s.test)?;
            let body = print_statement(&s.body)?;
            Some(format!("while ({test}) {body}"))
        }
        Statement::ThrowStatement(s) => {
            Some(format!("throw {}", print_expression(&s.argument)?))
        }
        Statement::BreakStatement(s) => Some(match s.label.as_deref() {
            Some(label) => format!("break {label}"),
            None => "break".into(),
        }),
        Statement::ContinueStatement(s) => Some(match s.label.as_deref() {
            Some(label) => format!("continue {label}"),
            None => "continue".into(),
        }),
        Statement::ClassDeclaration(_)
        | Statement::Interface(_)
        | Statement::TypeAlias(_)
        | Statement::Enum(_)
        | Statement::DoWhileStatement(_)
        | Statement::ForStatement(_)
        | Statement::ForInStatement(_)
        | Statement::ForOfStatement(_)
        | Statement::SwitchStatement(_)
        | Statement::TryStatement(_)
        | Statement::Namespace(_) => None,
    }
}

fn print_variable(v: &VariableDeclaration) -> Option<String> {
    if !v.decorators.is_empty() || v.is_declare {
        return None;
    }
    let init = match &v.value {
        Some(e) => format!(" = {}", print_expression(e)?),
        None => String::new(),
    };
    let ty = match &v.ty {
        Some(t) => format!(": {}", super::expr::print_type_annotation(t)?),
        None => String::new(),
    };
    Some(format!("const {}{}{}", v.name, ty, init))
}

fn print_import(d: &ImportDeclaration) -> Option<String> {
    let type_kw = if d.is_type_only { "type " } else { "" };
    let module = print_string(&d.module_specifier);
    if d.specifiers.is_empty() {
        return Some(format!("import {type_kw}{module};"));
    }
    let mut parts = Vec::with_capacity(d.specifiers.len());
    for spec in &d.specifiers {
        parts.push(match spec {
            ImportSpecifier::Default(name) => name.clone(),
            ImportSpecifier::Namespace(name) => format!("* as {name}"),
            ImportSpecifier::Named { local, imported } if local == imported => local.clone(),
            ImportSpecifier::Named { local, imported } => format!("{imported} as {local}"),
        });
    }
    Some(format!("import {type_kw}{{ {} }} from {};", parts.join(", "), module))
}

fn print_export(d: &ExportDeclaration) -> Option<String> {
    if !d.specifiers.is_empty() || d.source.is_some() || d.is_type_only {
        return None;
    }
    let default_kw = if d.is_default { "default " } else { "" };
    let decl = d.declaration.as_ref()?;
    let body = print_statement(decl)?;
    Some(format!("export {default_kw}{body}"))
}

fn print_function(d: &FunctionDeclaration) -> Option<String> {
    if !d.decorators.is_empty() || d.is_declare || !d.type_params.is_empty() {
        return None;
    }
    let params = print_function_params(&d.params)?;
    let ret = match &d.return_type {
        Some(t) => format!(": {}", super::expr::print_type_annotation(t)?),
        None => String::new(),
    };
    let body = print_block_body(&d.body)?;
    Some(format!("function {}{}{} {body}", d.name, params, ret))
}

fn print_function_params(params: &[crate::ast::FunctionParam]) -> Option<String> {
    if params.is_empty() {
        return Some("()".into());
    }
    let mut parts = Vec::with_capacity(params.len());
    for p in params {
        if !p.decorators.is_empty() {
            return None;
        }
        let ty = match &p.ty {
            Some(t) => format!(": {}", super::expr::print_type_annotation(t)?),
            None => String::new(),
        };
        let opt = if p.optional { "?" } else { "" };
        parts.push(format!("{}{}{}", p.name, opt, ty));
    }
    Some(format!("({})", parts.join(", ")))
}

fn print_block(b: &BlockStatement) -> Option<String> {
    print_block_body(&b.statements)
}

fn print_block_body(statements: &[Statement]) -> Option<String> {
    let mut parts = Vec::with_capacity(statements.len());
    for s in statements {
        parts.push(print_statement(s)?);
    }
    Some(format!("{{ {} }}", parts.join("; ")))
}
