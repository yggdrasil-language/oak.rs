//! Project-wide component ABI index and cross-file validation.

use std::collections::HashMap;

use super::{
    AbiIssue, AbiIssueKind, AbiSeverity, ComponentAbi,
    naming::{is_snake_case, normalize_event_name, normalize_prop_name},
    template::{TemplateBinding, TemplateBindingKind, collect_template_bindings},
};
use crate::{AwslRoot, awsl_stem_from_component_tag};

/// Index of component ABIs keyed by widget name (`snake_case`).
#[derive(Debug, Clone, Default)]
pub struct ComponentAbiIndex {
    by_widget: HashMap<String, ComponentAbi>,
}

impl ComponentAbiIndex {
    /// Build an empty index.
    pub fn new() -> Self {
        Self::default()
    }

    /// Insert or replace one widget ABI.
    pub fn insert(&mut self, widget_name: impl Into<String>, abi: ComponentAbi) {
        self.by_widget.insert(widget_name.into(), abi);
    }

    /// Build an index from `(widget_name, abi)` pairs.
    pub fn from_entries(entries: impl IntoIterator<Item = (String, ComponentAbi)>) -> Self {
        let mut index = Self::new();
        for (name, abi) in entries {
            index.insert(name, abi);
        }
        index
    }

    /// Lookup ABI by widget name.
    pub fn get(&self, widget_name: &str) -> Option<&ComponentAbi> {
        self.by_widget.get(widget_name)
    }

    /// Validate template bindings in one AWSL root against the index.
    pub fn validate_awsl_root(&self, root: &AwslRoot) -> Vec<AbiIssue> {
        let mut issues = Vec::new();
        for binding in collect_template_bindings(root) {
            validate_binding(&binding, self, &mut issues);
        }
        issues
    }

    /// Validate explicit template bindings.
    pub fn validate_bindings(&self, bindings: &[TemplateBinding]) -> Vec<AbiIssue> {
        let mut issues = Vec::new();
        for binding in bindings {
            validate_binding(binding, self, &mut issues);
        }
        issues
    }
}

fn validate_binding(binding: &TemplateBinding, index: &ComponentAbiIndex, issues: &mut Vec<AbiIssue>) {
    let target = awsl_stem_from_component_tag(&binding.component_tag);
    if !is_snake_case(&binding.name) {
        issues.push(AbiIssue {
            kind: AbiIssueKind::NotSnakeCase,
            message: format!("Name '{}' should be snake_case", binding.name),
            span: Some(binding.key_span.clone()),
            severity: AbiSeverity::Warning,
        });
    }
    let Some(abi) = index.get(&target)
    else {
        return;
    };
    match binding.kind {
        TemplateBindingKind::Property if abi.property(&binding.name).is_none() => {
            issues.push(AbiIssue {
                kind: AbiIssueKind::PropertyOnNonLet,
                message: format!("`:{}` is not declared on component `{target}`", binding.name),
                span: Some(binding.key_span.clone()),
                severity: AbiSeverity::Error,
            });
        }
        TemplateBindingKind::Event if abi.event(&binding.name).is_none() => {
            issues.push(AbiIssue {
                kind: AbiIssueKind::EventOnNonMicro,
                message: format!("`@{}` is not declared on component `{target}`", binding.name),
                span: Some(binding.key_span.clone()),
                severity: AbiSeverity::Error,
            });
        }
        _ => {}
    }
}

/// Refine whether a lowered attribute is ABI prop/event using the index.
pub fn refine_binding_kind(component_tag: &str, attr_name: &str, is_event_hint: bool, index: &ComponentAbiIndex) -> (bool, bool) {
    let target = awsl_stem_from_component_tag(component_tag);
    let Some(abi) = index.get(&target)
    else {
        return (false, is_event_hint);
    };
    if is_event_hint {
        let event = normalize_event_name(attr_name);
        (false, abi.event(&event).is_some())
    }
    else {
        let prop = normalize_prop_name(attr_name);
        (abi.property(&prop).is_some(), false)
    }
}
