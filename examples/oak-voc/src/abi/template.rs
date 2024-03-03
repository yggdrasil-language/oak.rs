//! Traverse AWSL template AST for component `:prop` / `@event` bindings.

use std::ops::Range;

use super::naming::{normalize_event_name, normalize_prop_name};
use crate::{AwslDirectiveKind, AwslElement, AwslRoot, AwslTemplateNode};

/// Kind of ABI template binding on a PascalCase child component.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateBindingKind {
    /// `:prop` input binding.
    Property,
    /// `@event` output binding.
    Event,
}

/// One `:prop` or `@event` usage on a child component tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateBinding {
    /// PascalCase component tag.
    pub component_tag: String,
    /// Normalized snake_case binding name.
    pub name: String,
    /// Property vs event.
    pub kind: TemplateBindingKind,
    /// Span of the binding key (`:checked`, `@change`, …).
    pub key_span: Range<usize>,
    /// Span of the enclosing element.
    pub element_span: Range<usize>,
}

/// Collect all ABI template bindings under `root.template`.
pub fn collect_template_bindings(root: &AwslRoot) -> Vec<TemplateBinding> {
    let mut out = Vec::new();
    for node in &root.template {
        collect_node_bindings(node, &mut out);
    }
    out
}

/// Find the template binding whose key span contains `offset`.
pub fn find_template_binding_at(root: &AwslRoot, offset: usize) -> Option<TemplateBinding> {
    collect_template_bindings(root).into_iter().find(|binding| binding.key_span.contains(&offset))
}

fn collect_node_bindings(node: &AwslTemplateNode, out: &mut Vec<TemplateBinding>) {
    match node {
        AwslTemplateNode::Element(element) => {
            if element.is_component() {
                collect_element_bindings(element, out);
            }
            for child in &element.children {
                collect_node_bindings(child, out);
            }
        }
        AwslTemplateNode::Text { .. } | AwslTemplateNode::Interpolation { .. } => {}
    }
}

fn collect_element_bindings(element: &AwslElement, out: &mut Vec<TemplateBinding>) {
    for attr in &element.attributes {
        let name = normalize_prop_name(&attr.name);
        let key_start = attr.span.start;
        out.push(TemplateBinding { component_tag: element.tag.clone(), name, kind: TemplateBindingKind::Property, key_span: key_start..key_start + attr.name.len() + 1, element_span: element.span.clone() });
    }
    for directive in &element.directives {
        if let Some(binding) = directive_to_binding(element, directive) {
            out.push(binding);
        }
    }
}

fn directive_to_binding(element: &AwslElement, directive: &crate::AwslDirective) -> Option<TemplateBinding> {
    let (event_name, key_len) = match &directive.kind {
        AwslDirectiveKind::On(name) if !is_non_abi_directive(name) => (normalize_event_name(name), name.len() + 1),
        _ => return None,
    };
    let key_start = directive.span.start;
    Some(TemplateBinding { component_tag: element.tag.clone(), name: event_name, kind: TemplateBindingKind::Event, key_span: key_start..key_start + key_len, element_span: element.span.clone() })
}

fn is_non_abi_directive(name: &str) -> bool {
    matches!(name, "if" | "loop" | "bind" | "style" | "ref" | "class" | "for")
}
