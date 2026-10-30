//! Oak TypeScript statement → canonical text.

use crate::ast::{
    BlockStatement, ClassDeclaration, ClassMember, DoWhileStatement, ExportDeclaration, ExportSpecifier, FunctionDeclaration, ImportDeclaration, ImportSpecifier, ModifierKind, Statement, TryStatement, TypeAliasDeclaration, VariableDeclaration,
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
        Statement::TypeAlias(d) => print_type_alias(d),
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
        Statement::DoWhileStatement(s) => print_do_while(s),
        Statement::TryStatement(s) => print_try(s),
        Statement::ThrowStatement(s) => Some(format!("throw {}", print_expression(&s.argument)?)),
        Statement::BreakStatement(s) => Some(match s.label.as_deref() {
            Some(label) => format!("break {label}"),
            None => "break".into(),
        }),
        Statement::ContinueStatement(s) => Some(match s.label.as_deref() {
            Some(label) => format!("continue {label}"),
            None => "continue".into(),
        }),
        Statement::ClassDeclaration(d) => print_class(d),
        Statement::Interface(_) | Statement::Enum(_) | Statement::ForStatement(_) | Statement::ForInStatement(_) | Statement::ForOfStatement(_) | Statement::SwitchStatement(_) | Statement::Namespace(_) => None,
    }
}

fn print_type_alias(d: &TypeAliasDeclaration) -> Option<String> {
    let ty = super::expr::print_type_annotation(&d.ty)?;
    Some(format!("type {} = {};", d.name, ty))
}

fn print_do_while(s: &DoWhileStatement) -> Option<String> {
    let body = print_statement(&s.body)?;
    let test = print_expression(&s.test)?;
    Some(format!("do {body} while ({test})"))
}

fn print_try(s: &TryStatement) -> Option<String> {
    let mut output = format!("try {}", print_block_body(&s.block)?);
    if let Some(handler) = &s.handler {
        let param = handler.param.as_deref().map(|name| format!(" ({name})")).unwrap_or_default();
        output.push_str(&format!(" catch{param} {}", print_block_body(&handler.body)?));
    }
    if let Some(finalizer) = &s.finalizer {
        output.push_str(&format!(" finally {}", print_block_body(finalizer)?));
    }
    Some(output)
}

fn print_variable(v: &VariableDeclaration) -> Option<String> {
    if !v.decorators.is_empty() || v.is_declare() {
        return None;
    }
    let mut parts = Vec::with_capacity(v.declarators.len());
    for declarator in &v.declarators {
        let binding = match &declarator.pattern_text {
            Some(text) => text.clone(),
            None => declarator.binding_names.first().cloned().unwrap_or_default(),
        };
        let init = match &declarator.value {
            Some(e) => format!(" = {}", print_expression(e)?),
            None => String::new(),
        };
        let ty = match &declarator.ty {
            Some(t) => format!(": {}", super::expr::print_type_annotation(t)?),
            None => String::new(),
        };
        parts.push(format!("{binding}{ty}{init}"));
    }
    Some(format!("const {}", parts.join(", ")))
}

fn print_import(d: &ImportDeclaration) -> Option<String> {
    let type_kw = if d.is_type_only() { "type " } else { "" };
    let module = print_string(&d.module_specifier.value);
    if d.specifiers.is_empty() {
        return Some(format!("import {type_kw}{module};"));
    }
    let mut parts = Vec::with_capacity(d.specifiers.len());
    for spec in &d.specifiers {
        parts.push(match spec {
            ImportSpecifier::Default { local, .. } => local.clone(),
            ImportSpecifier::Namespace { local, .. } => format!("* as {local}"),
            ImportSpecifier::Named { local, imported, type_modifier, .. } => {
                let type_kw = if type_modifier.is_some() { "type " } else { "" };
                if local == imported { format!("{type_kw}{local}") } else { format!("{type_kw}{imported} as {local}") }
            }
        });
    }
    Some(format!("import {type_kw}{{ {} }} from {};", parts.join(", "), module))
}

fn print_export(d: &ExportDeclaration) -> Option<String> {
    if !d.specifiers.is_empty() {
        let source = d.source.as_ref().map(|value| format!(" from {}", print_string(&value.value))).unwrap_or_default();
        let type_kw = if d.is_type_only() { "type " } else { "" };
        if d.specifiers.len() == 1 {
            match &d.specifiers[0] {
                ExportSpecifier::All { .. } => return Some(format!("export {type_kw}*{source};")),
                ExportSpecifier::Namespace { exported, .. } => return Some(format!("export {type_kw}* as {exported}{source};")),
                ExportSpecifier::Named { .. } => {}
            }
        }
        let mut parts = Vec::with_capacity(d.specifiers.len());
        for specifier in &d.specifiers {
            let ExportSpecifier::Named { local, exported, type_modifier, .. } = specifier
            else {
                return None;
            };
            let type_kw = if type_modifier.is_some() { "type " } else { "" };
            let name = if local == exported { local.clone() } else { format!("{local} as {exported}") };
            parts.push(format!("{type_kw}{name}"));
        }
        return Some(format!("export {type_kw}{{ {} }}{source};", parts.join(", ")));
    }
    if d.source.is_some() || d.is_type_only() {
        return None;
    }
    let default_kw = if d.is_default() { "default " } else { "" };
    let decl = d.declaration.as_ref()?;
    let body = print_statement(decl)?;
    Some(format!("export {default_kw}{body}"))
}

fn print_function(d: &FunctionDeclaration) -> Option<String> {
    if !d.decorators.is_empty() || d.is_declare() || !d.type_params.is_empty() {
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
        let opt = if p.is_optional() { "?" } else { "" };
        parts.push(format!("{}{}{}", p.name, opt, ty));
    }
    Some(format!("({})", parts.join(", ")))
}

fn print_class(d: &ClassDeclaration) -> Option<String> {
    if !d.decorators.is_empty() || d.is_declare() || d.is_abstract() || !d.type_params.is_empty() || !d.implements.is_empty() {
        return None;
    }
    let extends = match d.extends.as_ref() {
        Some(ty) => format!(" extends {}", super::expr::print_type_annotation(ty)?),
        None => String::new(),
    };
    let mut members = Vec::with_capacity(d.body.len());
    for member in &d.body {
        members.push(print_class_member(member)?);
    }
    let body = if members.is_empty() { String::new() } else { format!(" {} ", members.join(" ")) };
    Some(format!("class {}{} {{{body}}}", d.name, extends))
}

fn print_class_member(member: &ClassMember) -> Option<String> {
    match member {
        ClassMember::Property { decorators, name, initializer, optional_marker, .. } => {
            if !decorators.is_empty() || member.has_modifier(ModifierKind::Readonly) || member.has_modifier(ModifierKind::Abstract) || optional_marker.is_some() {
                return None;
            }
            let static_kw = if member.has_modifier(ModifierKind::Static) { "static " } else { "" };
            let initializer = match initializer.as_ref() {
                Some(expr) => format!(" = {}", print_expression(expr)?),
                None => String::new(),
            };
            Some(format!("{static_kw}{name}{initializer};"))
        }
        ClassMember::Method { decorators, name, params, body, optional_marker, type_params, .. } => {
            if !decorators.is_empty() || member.has_modifier(ModifierKind::Abstract) || optional_marker.is_some() || !type_params.is_empty() {
                return None;
            }
            let static_kw = if member.has_modifier(ModifierKind::Static) { "static " } else { "" };
            let async_kw = if member.has_modifier(ModifierKind::Async) { "async " } else { "" };
            let accessor = if member.has_modifier(ModifierKind::Get) {
                "get "
            }
            else if member.has_modifier(ModifierKind::Set) {
                "set "
            }
            else {
                ""
            };
            let params = print_function_params(params)?;
            let body = print_block_body(body)?;
            Some(format!("{static_kw}{async_kw}{accessor}{name}{params} {body}"))
        }
    }
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
