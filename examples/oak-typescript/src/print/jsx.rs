//! Oak TypeScript JSX → canonical text.

use crate::ast::{Expression, JsxAttribute, JsxAttributeOrSpread, JsxAttributeValue, JsxChild, JsxClosingElement, JsxElement, JsxFragment, JsxOpeningElement, JsxSelfClosingElement, JsxTagName};

use super::expr::{print_expression, print_string};

pub fn print_jsx_element(element: &JsxElement) -> Option<String> {
    let open = print_jsx_opening(&element.opening_element)?;
    let children = print_jsx_children(&element.children)?;
    let close = print_jsx_closing(&element.closing_element)?;
    Some(format!("{open}{children}{close}"))
}

pub fn print_jsx_self_closing(element: &JsxSelfClosingElement) -> Option<String> {
    let name = print_jsx_tag_name(&element.name)?;
    let attrs = print_jsx_attributes(&element.attributes)?;
    Some(format!("<{name}{attrs} />"))
}

pub fn print_jsx_fragment(fragment: &JsxFragment) -> Option<String> {
    let children = print_jsx_children(&fragment.children)?;
    Some(format!("<>{children}</>"))
}

fn print_jsx_opening(opening: &JsxOpeningElement) -> Option<String> {
    let name = print_jsx_tag_name(&opening.name)?;
    let attrs = print_jsx_attributes(&opening.attributes)?;
    Some(format!("<{name}{attrs}>"))
}

fn print_jsx_closing(closing: &JsxClosingElement) -> Option<String> {
    let name = print_jsx_tag_name(&closing.name)?;
    Some(format!("</{name}>"))
}

fn print_jsx_tag_name(name: &JsxTagName) -> Option<String> {
    match name {
        JsxTagName::Identifier(id) => Some(id.trim().to_string()),
        JsxTagName::MemberExpression { object, property } => Some(format!("{}.{property}", print_jsx_tag_name(object)?)),
    }
}

fn print_jsx_attributes(attributes: &[JsxAttributeOrSpread]) -> Option<String> {
    let mut parts = Vec::with_capacity(attributes.len());
    for attr in attributes {
        if let Some(text) = print_jsx_attribute_or_spread(attr) {
            if !text.is_empty() {
                parts.push(text);
            }
        }
    }
    if parts.is_empty() {
        return Some(String::new());
    }
    Some(format!(" {}", parts.join(" ")))
}

fn print_jsx_attribute_or_spread(attr: &JsxAttributeOrSpread) -> Option<String> {
    match attr {
        JsxAttributeOrSpread::Attribute(attribute) => print_jsx_attribute(attribute),
        JsxAttributeOrSpread::Spread(expression) => Some(format!("{{...{}}}", print_expression(expression)?)),
    }
}

fn print_jsx_attribute(attribute: &JsxAttribute) -> Option<String> {
    let name = attribute.name.trim();
    if name.is_empty() {
        return None;
    }
    match &attribute.value {
        None => Some(name.to_string()),
        Some(value) => Some(format!("{}={}", name, print_jsx_attribute_value(value)?)),
    }
}

fn decode_jsx_string_literal(raw: &str) -> String {
    raw.strip_prefix('"').and_then(|s| s.strip_suffix('"')).or_else(|| raw.strip_prefix('\'').and_then(|s| s.strip_suffix('\''))).map(str::to_string).unwrap_or_else(|| raw.to_string())
}

fn print_jsx_attribute_value(value: &JsxAttributeValue) -> Option<String> {
    match value {
        JsxAttributeValue::StringLiteral(s) => Some(print_string(&decode_jsx_string_literal(s))),
        JsxAttributeValue::ExpressionContainer(expr) => print_jsx_expression_container(expr),
        JsxAttributeValue::Element(element) => Some(print_jsx_element(element)?),
        JsxAttributeValue::Fragment(fragment) => Some(print_jsx_fragment(fragment)?),
    }
}

fn print_jsx_children(children: &[JsxChild]) -> Option<String> {
    let mut out = String::new();
    for child in children {
        out.push_str(&print_jsx_child(child)?);
    }
    Some(out)
}

fn print_jsx_child(child: &JsxChild) -> Option<String> {
    match child {
        JsxChild::JsxElement(element) => print_jsx_element(element),
        JsxChild::JsxFragment(fragment) => print_jsx_fragment(fragment),
        JsxChild::JsxSelfClosingElement(element) => print_jsx_self_closing(element),
        JsxChild::JsxText(text) => Some(text.clone()),
        JsxChild::JsxExpressionContainer(expr) => print_jsx_expression_container(expr),
    }
}

fn print_jsx_expression_container(expr: &Option<Expression>) -> Option<String> {
    match expr {
        Some(expression) => Some(format!("{{{}}}", print_expression(expression)?)),
        None => Some("{}".into()),
    }
}
