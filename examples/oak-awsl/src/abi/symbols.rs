//! ABI symbol references for IDE navigation (LSP / JetBrains).

use std::ops::Range;

use super::{
    AbiEvent, AbiProperty, ComponentAbi,
    template::{TemplateBinding, TemplateBindingKind, collect_template_bindings},
};
use crate::AwslRoot;

/// Classified ABI symbol kind for navigation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiSymbolKind {
    /// `[property] let` / template `:prop`.
    Property,
    /// `[event] micro` / template `@event`.
    Event,
    /// `emit(event, …)` target.
    EmitTarget,
    /// `effect(...)` call site.
    Effect,
    /// `[memoize] let`.
    Memo,
}

/// One ABI reference occurrence in a single file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiReference {
    /// Symbol kind.
    pub kind: AbiSymbolKind,
    /// Normalized snake_case symbol name.
    pub name: String,
    /// For template bindings: target PascalCase component tag.
    pub component_tag: Option<String>,
    /// Byte range in the AWSL source file.
    pub span: Range<usize>,
}

/// Collect ABI references from script ABI + template usages in one file.
pub fn collect_abi_references(component_abi: &ComponentAbi, _script_source: &str, root: &AwslRoot) -> Vec<AbiReference> {
    let mut refs = Vec::new();
    for property in &component_abi.properties {
        refs.push(AbiReference {
            kind: AbiSymbolKind::Property,
            name: property.name.clone(),
            component_tag: None,
            span: property.span.clone(),
        });
    }
    for event in &component_abi.events {
        refs.push(AbiReference { kind: AbiSymbolKind::Event, name: event.name.clone(), component_tag: None, span: event.span.clone() });
    }
    for memo in &component_abi.memoized {
        refs.push(AbiReference { kind: AbiSymbolKind::Memo, name: memo.name.clone(), component_tag: None, span: memo.span.clone() });
    }
    for effect in &component_abi.effects {
        refs.push(AbiReference { kind: AbiSymbolKind::Effect, name: "effect".into(), component_tag: None, span: effect.span.clone() });
    }
    for binding in collect_template_bindings(root) {
        let kind = match binding.kind {
            TemplateBindingKind::Property => AbiSymbolKind::Property,
            TemplateBindingKind::Event => AbiSymbolKind::Event,
        };
        refs.push(AbiReference {
            kind,
            name: binding.name.clone(),
            component_tag: Some(binding.component_tag.clone()),
            span: binding.key_span.clone(),
        });
    }
    refs
}

/// Find ABI declaration span for a property or event name.
pub fn abi_declaration_span(abi: &ComponentAbi, kind: AbiSymbolKind, name: &str) -> Option<Range<usize>> {
    match kind {
        AbiSymbolKind::Property => abi.property(name).map(|p: &AbiProperty| p.span.clone()),
        AbiSymbolKind::Event | AbiSymbolKind::EmitTarget => abi.event(name).map(|e: &AbiEvent| e.span.clone()),
        AbiSymbolKind::Memo => abi.memoized.iter().find(|m| m.name == name).map(|m| m.span.clone()),
        AbiSymbolKind::Effect => None,
    }
}

/// Classify cursor offset against collected references and declarations.
pub fn classify_abi_cursor(abi: &ComponentAbi, root: &AwslRoot, offset: usize) -> Option<(AbiSymbolKind, String, Option<String>)> {
    for binding in collect_template_bindings(root) {
        if binding.key_span.contains(&offset) {
            let kind = match binding.kind {
                TemplateBindingKind::Property => AbiSymbolKind::Property,
                TemplateBindingKind::Event => AbiSymbolKind::Event,
            };
            return Some((kind, binding.name.clone(), Some(binding.component_tag.clone())));
        }
    }
    if let Some(prop) = abi.properties.iter().find(|p| p.span.contains(&offset)) {
        return Some((AbiSymbolKind::Property, prop.name.clone(), None));
    }
    if let Some(event) = abi.events.iter().find(|e| e.span.contains(&offset)) {
        return Some((AbiSymbolKind::Event, event.name.clone(), None));
    }
    None
}
