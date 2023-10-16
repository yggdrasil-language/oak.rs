//! Oak TypeScript expression → canonical text.

use crate::ast::{
    Expression, ExpressionKind, FunctionParam, ObjectProperty, Statement, TypeAnnotation,
};

/// Print one Oak expression node.
pub fn print_expression(expr: &Expression) -> Option<String> {
    print_expr(expr, Prec::Lowest)
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Prec {
    Lowest = 0,
    Assign = 1,
    Ternary = 2,
    Or = 3,
    And = 4,
    BitOr = 5,
    BitXor = 6,
    BitAnd = 7,
    Eq = 8,
    Rel = 9,
    Shift = 10,
    Add = 11,
    Mul = 12,
    Exp = 13,
    Unary = 14,
    Postfix = 15,
    Call = 16,
    Primary = 17,
}

fn print_expr(expr: &Expression, min_prec: Prec) -> Option<String> {
    match expr.kind.as_ref() {
        ExpressionKind::Identifier(name) => Some(name.clone()),
        ExpressionKind::NumericLiteral(n) => Some(print_number(*n)),
        ExpressionKind::StringLiteral(s) => Some(print_string(s)),
        ExpressionKind::BigIntLiteral(s) => Some(format!("{s}n")),
        ExpressionKind::BooleanLiteral(b) => Some(if *b { "true" } else { "false" }.into()),
        ExpressionKind::NullLiteral => Some("null".into()),
        ExpressionKind::RegexLiteral(s) => Some(s.clone()),
        ExpressionKind::TemplateString(s) => Some(format!("`{s}`")),
        ExpressionKind::UnaryExpression { operator, argument } => {
            let inner = print_expr(argument, Prec::Unary)?;
            let text = if operator == "typeof" || operator == "void" || operator == "delete" {
                format!("{operator} {inner}")
            } else {
                format!("{operator}{inner}")
            };
            wrap_prec(text, Prec::Unary, min_prec)
        }
        ExpressionKind::UpdateExpression { operator, argument, prefix } => {
            let inner = print_expr(argument, Prec::Postfix)?;
            let text =
                if *prefix { format!("{operator}{inner}") } else { format!("{inner}{operator}") };
            wrap_prec(text, Prec::Postfix, min_prec)
        }
        ExpressionKind::BinaryExpression { left, operator, right } => {
            let (prec, right_prec) = binary_prec(operator)?;
            let left_s = if operator == "**" {
                print_expr(left, Prec::Unary)?
            } else {
                print_expr(left, prec)?
            };
            let right_s = print_expr(right, right_prec)?;
            let spaced = if operator == "," {
                format!("{left_s},{right_s}")
            } else {
                format!("{left_s} {operator} {right_s}")
            };
            wrap_prec(spaced, prec, min_prec)
        }
        ExpressionKind::ConditionalExpression { test, consequent, alternate } => {
            let test_s = print_expr(test, Prec::Or)?;
            let cons_s = print_expr(consequent, Prec::Assign)?;
            let alt_s = print_expr(alternate, Prec::Assign)?;
            wrap_prec(format!("{test_s} ? {cons_s} : {alt_s}"), Prec::Ternary, min_prec)
        }
        ExpressionKind::MemberExpression { object, property, computed, optional } => {
            let obj = print_expr(object, Prec::Call)?;
            let opt = if *optional { "?." } else { "." };
            let prop = if *computed {
                format!("[{}]", print_expr(property, Prec::Lowest)?)
            } else if let ExpressionKind::Identifier(name) = property.kind.as_ref() {
                name.clone()
            } else {
                format!("[{}]", print_expr(property, Prec::Lowest)?)
            };
            wrap_prec(format!("{obj}{opt}{prop}"), Prec::Call, min_prec)
        }
        ExpressionKind::CallExpression { func, args } => {
            if matches!(func.kind.as_ref(), ExpressionKind::Identifier(n) if n == "import") {
                let arg = args.first()?;
                return Some(format!("import({})", print_expr(arg, Prec::Lowest)?));
            }
            if args.is_empty() {
                if let ExpressionKind::StringLiteral(spec) = func.kind.as_ref() {
                    return Some(format!("import({})", print_string(spec)));
                }
            }
            let callee = print_expr(func, Prec::Call)?;
            let mut parts = Vec::with_capacity(args.len());
            for a in args {
                parts.push(print_expr(a, Prec::Assign)?);
            }
            wrap_prec(format!("{callee}({})", parts.join(", ")), Prec::Call, min_prec)
        }
        ExpressionKind::NewExpression { func, args } => {
            let callee = print_expr(func, Prec::Call)?;
            let mut parts = Vec::with_capacity(args.len());
            for a in args {
                parts.push(print_expr(a, Prec::Assign)?);
            }
            wrap_prec(format!("new {callee}({})", parts.join(", ")), Prec::Call, min_prec)
        }
        ExpressionKind::AssignmentExpression { left, operator, right } => {
            let left_s = print_expr(left, Prec::Assign)?;
            let right_s = print_expr(right, Prec::Assign)?;
            wrap_prec(format!("{left_s} {operator} {right_s}"), Prec::Assign, min_prec)
        }
        ExpressionKind::ArrayLiteral { elements } => {
            let mut parts = Vec::with_capacity(elements.len());
            for e in elements {
                parts.push(print_expr(e, Prec::Assign)?);
            }
            wrap_prec(format!("[{}]", parts.join(", ")), Prec::Primary, min_prec)
        }
        ExpressionKind::ObjectLiteral { properties } => {
            let mut parts = Vec::with_capacity(properties.len());
            for p in properties {
                parts.push(print_object_prop(p)?);
            }
            wrap_prec(format!("{{ {} }}", parts.join(", ")), Prec::Primary, min_prec)
        }
        ExpressionKind::SpreadElement(inner) => {
            wrap_prec(format!("...{}", print_expr(inner, Prec::Unary)?), Prec::Unary, min_prec)
        }
        ExpressionKind::AwaitExpression(inner) => {
            wrap_prec(format!("await {}", print_expr(inner, Prec::Unary)?), Prec::Unary, min_prec)
        }
        ExpressionKind::YieldExpression(arg) => {
            let text = match arg {
                Some(inner) => format!("yield {}", print_expr(inner, Prec::Unary)?),
                None => "yield".into(),
            };
            wrap_prec(text, Prec::Unary, min_prec)
        }
        ExpressionKind::ImportExpression { module_specifier } => wrap_prec(
            format!("import({})", print_expr(module_specifier, Prec::Lowest)?),
            Prec::Call,
            min_prec,
        ),
        ExpressionKind::ArrowFunction { params, body, async_, .. } => {
            let prefix = if *async_ { "async " } else { "" };
            let params_s = print_params(params)?;
            let body_s = print_arrow_body(body)?;
            wrap_prec(format!("{prefix}{params_s} => {body_s}"), Prec::Assign, min_prec)
        }
        ExpressionKind::AsExpression { expression, type_annotation }
        | ExpressionKind::TypeAssertionExpression { expression, type_annotation } => {
            let expr_s = print_expr(expression, Prec::Unary)?;
            let ty = print_type_annotation(type_annotation)?;
            wrap_prec(format!("{expr_s} as {ty}"), Prec::Unary, min_prec)
        }
        ExpressionKind::NonNullExpression(inner) => {
            wrap_prec(format!("{}!", print_expr(inner, Prec::Postfix)?), Prec::Postfix, min_prec)
        }
        ExpressionKind::FunctionExpression { .. } | ExpressionKind::TaggedTemplateExpression { .. } => None,
        ExpressionKind::JsxElement(element) => {
            wrap_prec(super::jsx::print_jsx_element(element)?, Prec::Primary, min_prec)
        }
        ExpressionKind::JsxFragment(fragment) => {
            wrap_prec(super::jsx::print_jsx_fragment(fragment)?, Prec::Primary, min_prec)
        }
        ExpressionKind::JsxSelfClosingElement(element) => {
            wrap_prec(super::jsx::print_jsx_self_closing(element)?, Prec::Primary, min_prec)
        }
    }
}

fn wrap_prec(text: String, prec: Prec, min_prec: Prec) -> Option<String> {
    if prec < min_prec { Some(format!("({text})")) } else { Some(text) }
}

fn binary_prec(op: &str) -> Option<(Prec, Prec)> {
    match op {
        "||" | "??" => Some((Prec::Or, Prec::Or)),
        "&&" => Some((Prec::And, Prec::And)),
        "|" => Some((Prec::BitOr, Prec::BitOr)),
        "^" => Some((Prec::BitXor, Prec::BitXor)),
        "&" => Some((Prec::BitAnd, Prec::BitAnd)),
        "==" | "!=" | "===" | "!==" => Some((Prec::Eq, Prec::Eq)),
        "<" | "<=" | ">" | ">=" | "in" | "instanceof" => Some((Prec::Rel, Prec::Rel)),
        "<<" | ">>" | ">>>" => Some((Prec::Shift, Prec::Shift)),
        "+" | "-" => Some((Prec::Add, Prec::Add)),
        "*" | "/" | "%" => Some((Prec::Mul, Prec::Mul)),
        "**" => Some((Prec::Exp, Prec::Exp)),
        "," => Some((Prec::Assign, Prec::Assign)),
        _ => None,
    }
}

fn print_object_prop(prop: &ObjectProperty) -> Option<String> {
    match prop {
        ObjectProperty::Property { name, value, shorthand, .. } => {
            if *shorthand {
                return Some(name.clone());
            }
            Some(format!("{}: {}", name, print_expr(value, Prec::Assign)?))
        }
        ObjectProperty::Spread(expr) => Some(format!("...{}", print_expr(expr, Prec::Unary)?)),
    }
}

fn print_params(params: &[FunctionParam]) -> Option<String> {
    if params.is_empty() {
        return Some("()".into());
    }
    if params.len() == 1 && params[0].name.is_empty() {
        return None;
    }
    let mut parts = Vec::with_capacity(params.len());
    for p in params {
        if p.name.is_empty() {
            return None;
        }
        parts.push(p.name.clone());
    }
    Some(format!("({})", parts.join(", ")))
}

fn print_arrow_body(body: &Statement) -> Option<String> {
    match body {
        Statement::ExpressionStatement(es) => print_expr(&es.expression, Prec::Assign),
        Statement::BlockStatement(b) => {
            let mut parts = Vec::new();
            for s in &b.statements {
                parts.push(super::stmt::print_statement(s)?);
            }
            Some(format!("{{ {} }}", parts.join("; ")))
        }
        _ => None,
    }
}

pub(super) fn print_type_annotation(ty: &TypeAnnotation) -> Option<String> {
    match ty {
        TypeAnnotation::Identifier(name) | TypeAnnotation::Predefined(name) => Some(name.clone()),
        TypeAnnotation::Literal(lit) => match lit {
            crate::ast::LiteralType::String(s) => Some(print_string(s)),
            crate::ast::LiteralType::Number(n) => Some(print_number(*n)),
            crate::ast::LiteralType::Boolean(b) => Some(if *b { "true" } else { "false" }.into()),
            crate::ast::LiteralType::BigInt(s) => Some(format!("{s}n")),
        },
        _ => None,
    }
}

pub(super) fn print_string(s: &str) -> String {
    if !s.contains('\'') {
        format!("'{}'", escape_js(s, '\''))
    } else if !s.contains('"') {
        format!("\"{}\"", escape_js(s, '"'))
    } else {
        format!("\"{}\"", escape_js(s, '"'))
    }
}

fn escape_js(s: &str, quote: char) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c == quote => {
                out.push('\\');
                out.push(c);
            }
            c => out.push(c),
        }
    }
    out
}

fn print_number(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        n.to_string()
    }
}

#[cfg(test)]
mod tests {
    use oak_core::{Builder, ParseSession, SourceText};

    use crate::ast::{ExpressionKind, Statement};
    use crate::{TypeScriptBuilder, TypeScriptLanguage};

    use super::*;

    fn parse_expr_snippet(trimmed: &str) -> Option<Expression> {
        let source = SourceText::new(trimmed);
        let language = TypeScriptLanguage::default();
        let builder = TypeScriptBuilder::new(&language);
        let mut cache = ParseSession::default();
        let built = Builder::build(&builder, &source, &[], &mut cache);
        built.result.ok().and_then(|root| {
            root.statements.iter().find_map(|stmt| match stmt {
                Statement::ExpressionStatement(es) => Some(es.expression.clone()),
                _ => None,
            })
        })
    }

    #[test]
    fn prints_binary_with_spacing() {
        let expr = parse_expr_snippet("a+b").expect("parse");
        assert_eq!(print_expression(&expr).as_deref(), Some("a + b"));
    }

    #[test]
    fn prints_ternary_string_literals() {
        let expr = parse_expr_snippet(r#"error ? "true" : "false""#).expect("parse");
        match expr.kind.as_ref() {
            ExpressionKind::ConditionalExpression { consequent, alternate, .. } => {
                match consequent.kind.as_ref() {
                    ExpressionKind::StringLiteral(s) => assert_eq!(s, "true"),
                    other => panic!("consequent: {other:?}"),
                }
                match alternate.kind.as_ref() {
                    ExpressionKind::StringLiteral(s) => assert_eq!(s, "false"),
                    other => panic!("alternate: {other:?}"),
                }
            }
            other => panic!("root: {other:?}"),
        }
        assert_eq!(
            print_expression(&expr).as_deref(),
            Some("error ? 'true' : 'false'")
        );
    }
}
