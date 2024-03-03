//! AWSL component ABI extracted from `[property]` / `[event]` / `[memoize]` attributes.

mod extract;
mod index;
mod naming;
mod symbols;
mod template;

pub use extract::{AbiExtractResult, AbiIssue, AbiIssueKind, AbiSeverity, extract_component_abi_from_script, extract_component_abi_from_vx};
pub use index::{ComponentAbiIndex, refine_binding_kind};
pub use naming::{is_snake_case, normalize_event_name, normalize_prop_name};
pub use symbols::{AbiReference, AbiSymbolKind, abi_declaration_span, classify_abi_cursor, collect_abi_references};
pub use template::{TemplateBinding, TemplateBindingKind, collect_template_bindings, find_template_binding_at};

use std::ops::Range;

/// Component input/output ABI for a single widget.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ComponentAbi {
    /// Widget name (`snake_case`).
    pub widget_name: String,
    /// `[property] let` inputs.
    pub properties: Vec<AbiProperty>,
    /// `[event] micro` outputs.
    pub events: Vec<AbiEvent>,
    /// `let mut` internal reactive state.
    pub states: Vec<AbiState>,
    /// `let = expr` derived bindings.
    pub derived: Vec<AbiDerived>,
    /// `[memoize] let = expr` cached derived bindings.
    pub memoized: Vec<AbiMemo>,
    /// `effect(...)` side-effect blocks (for unused-dep warnings).
    pub effects: Vec<AbiEffect>,
}

/// `[property] let` component input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiProperty {
    /// Property name (`snake_case`).
    pub name: String,
    /// Optional type annotation source text.
    pub type_hint: Option<String>,
    /// Required when no initializer is present.
    pub required: bool,
    /// Optional default initializer source text.
    pub default_expr: Option<String>,
    /// Source span in the original script / synthetic widget.
    pub span: Range<usize>,
}

/// `[event] micro` component output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiEvent {
    /// Event name (`snake_case`).
    pub name: String,
    /// Event parameter names and type hints.
    pub params: Vec<AbiParam>,
    /// Source span of the micro declaration.
    pub span: Range<usize>,
}

/// Function / event parameter.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiParam {
    /// Parameter name.
    pub name: String,
    /// Optional type hint source text.
    pub type_hint: Option<String>,
}

/// `let mut` internal state.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiState {
    /// State variable name.
    pub name: String,
    /// Initial expression source text.
    pub init_expr: Option<String>,
    /// Source span.
    pub span: Range<usize>,
}

/// Auto-derived `let = expr` binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiDerived {
    /// Binding name.
    pub name: String,
    /// Expression source text.
    pub expr: String,
    /// Source span.
    pub span: Range<usize>,
}

/// `[memoize] let = expr` cached binding.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiMemo {
    /// Binding name.
    pub name: String,
    /// Expression source text.
    pub expr: String,
    /// Source span.
    pub span: Range<usize>,
}

/// `effect(deps) { ... }` side-effect statement.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiEffect {
    /// Dependency identifier names.
    pub deps: Vec<String>,
    /// Source span of the effect call.
    pub span: Range<usize>,
}

impl ComponentAbi {
    /// Lookup a declared property by name.
    pub fn property(&self, name: &str) -> Option<&AbiProperty> {
        self.properties.iter().find(|prop| prop.name == name)
    }

    /// Lookup a declared event by name.
    pub fn event(&self, name: &str) -> Option<&AbiEvent> {
        self.events.iter().find(|event| event.name == name)
    }
}

#[cfg(test)]
mod tests;
