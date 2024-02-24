//! 缩进多行 AST print（composite 类型折行，标量仍走 `ToSource`）。

use oak_core::source::{SourceBuffer, ToSource};

use crate::ast::VonValue;

use super::options::PrintOptions;

pub(crate) fn print_indented(value: &VonValue, depth: usize, options: &PrintOptions) -> String {
    match value {
        VonValue::Object(object) => print_object(object, depth, options),
        VonValue::Array(array) => print_array(array, depth, options),
        VonValue::Tuple(tuple) => print_tuple(tuple, depth, options),
        VonValue::Enum(enum_value) => print_enum(enum_value, depth, options),
        _ => {
            let mut buffer = SourceBuffer::new();
            value.to_source(&mut buffer);
            buffer.to_string()
        }
    }
}

fn print_object(object: &crate::ast::VonObject, depth: usize, options: &PrintOptions) -> String {
    if object.fields.is_empty() {
        return "{}".to_string();
    }
    let pad = " ".repeat(depth);
    let inner = depth + options.indent_width;
    let inner_pad = " ".repeat(inner);
    let body = object
        .fields
        .iter()
        .map(|field| {
            format!(
                "{}{}={}",
                inner_pad,
                field.name,
                print_indented(&field.value, inner, options)
            )
        })
        .collect::<Vec<_>>()
        .join(",\n");
    format!("{{\n{}\n{}}}", body, pad)
}

fn print_array(array: &crate::ast::VonArray, depth: usize, options: &PrintOptions) -> String {
    if array.elements.is_empty() {
        return "[]".to_string();
    }
    let pad = " ".repeat(depth);
    let inner = depth + options.indent_width;
    let inner_pad = " ".repeat(inner);
    let body = array
        .elements
        .iter()
        .map(|element| format!("{}{}", inner_pad, print_indented(element, inner, options)))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("[\n{}\n{}]", body, pad)
}

fn print_tuple(tuple: &crate::ast::VonTuple, depth: usize, options: &PrintOptions) -> String {
    if tuple.elements.is_empty() {
        return "()".to_string();
    }
    if tuple.elements.len() == 1 {
        let mut buffer = SourceBuffer::new();
        tuple.to_source(&mut buffer);
        return buffer.to_string();
    }
    let pad = " ".repeat(depth);
    let inner = depth + options.indent_width;
    let inner_pad = " ".repeat(inner);
    let body = tuple
        .elements
        .iter()
        .map(|element| format!("{}{}", inner_pad, print_indented(element, inner, options)))
        .collect::<Vec<_>>()
        .join(",\n");
    format!("(\n{}\n{})", body, pad)
}

fn print_enum(enum_value: &crate::ast::VonEnum, depth: usize, options: &PrintOptions) -> String {
    match &enum_value.payload {
        Some(payload)
            if matches!(payload.as_ref(), VonValue::Object(_) | VonValue::Array(_) | VonValue::Tuple(_)) =>
        {
            format!("{} {}", enum_value.variant, print_indented(payload, depth, options))
        }
        Some(payload) => {
            let mut buffer = SourceBuffer::new();
            buffer.push(&enum_value.variant);
            buffer.push(" ");
            payload.to_source(&mut buffer);
            buffer.to_string()
        }
        None => enum_value.variant.clone(),
    }
}
