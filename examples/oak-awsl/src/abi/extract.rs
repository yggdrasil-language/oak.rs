//! Extract AWSL component ABI from vx / widget AST (`oak-valkyrie`).

use std::ops::Range;

use oak_valkyrie::ast::{
    Attribute, Block, Let, MicroDeclaration, Pattern, StatementNode, StringLiteral, StringSegment, TermExpression, TypeExpression,
    ValkyrieRoot, WidgetDeclaration,
};

use crate::frontend::{parse_vx_source, std_range};

use super::{AbiDerived, AbiEffect, AbiEvent, AbiMemo, AbiParam, AbiProperty, AbiState, ComponentAbi};

/// Result of ABI extraction, including diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiExtractResult {
    /// Extracted ABI (may be partial when errors are present).
    pub abi: ComponentAbi,
    /// Semantic issues encountered during extraction.
    pub issues: Vec<AbiIssue>,
}

/// One ABI diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbiIssue {
    /// Issue classification.
    pub kind: AbiIssueKind,
    /// Human-readable message.
    pub message: String,
    /// Source span when available.
    pub span: Option<Range<usize>>,
    /// Warning vs hard error.
    pub severity: AbiSeverity,
}

/// ABI issue codes aligned with the RFC diagnostic table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiIssueKind {
    /// `[property]` on non-`let`.
    PropertyOnNonLet,
    /// `[event]` on non-`micro`.
    EventOnNonMicro,
    /// `[memoize]` on `let mut`.
    MemoizeOnMut,
    /// `[event] micro` has non-empty body.
    EventNonEmptyBody,
    /// `emit` target is not an event micro.
    EmitInvalidTarget,
    /// `emit` arity mismatch.
    EmitArityMismatch,
    /// Attribute combination forbidden.
    AttributeConflict,
    /// ABI name is not snake_case.
    NotSnakeCase,
    /// `[event] micro` declared but never emitted.
    EventNeverEmitted,
    /// `[property] let` never read.
    PropertyNeverRead,
    /// `[memoize]` on trivial expression.
    MemoizeTrivial,
    /// `effect` dependency unused in block.
    EffectUnusedDep,
    /// Legacy string `emit("name", ...)`.
    LegacyStringEmit,
}

/// Warning vs error severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AbiSeverity {
    /// Hard error.
    Error,
    /// Non-fatal warning.
    Warning,
}

/// Extract ABI from raw `<script>` text by wrapping it in a synthetic widget.
pub fn extract_component_abi_from_script(script: &str, widget_name: &str) -> AbiExtractResult {
    let synthetic = format!("widget {widget_name} {{\n{script}\n}}\n");
    extract_component_abi_from_vx(&synthetic, widget_name)
}

/// Extract ABI from vx source containing a widget declaration.
pub fn extract_component_abi_from_vx(source: &str, widget_name: &str) -> AbiExtractResult {
    let mut issues = Vec::new();
    let Ok(root) = parse_vx_source(source)
    else {
        return AbiExtractResult {
            abi: ComponentAbi { widget_name: widget_name.to_string(), ..Default::default() },
            issues: vec![AbiIssue {
                kind: AbiIssueKind::PropertyOnNonLet,
                message: "failed to parse script as vx".into(),
                span: None,
                severity: AbiSeverity::Error,
            }],
        };
    };

    let Some(widget) = find_widget(&root, widget_name)
    else {
        return AbiExtractResult {
            abi: ComponentAbi { widget_name: widget_name.to_string(), ..Default::default() },
            issues: vec![AbiIssue {
                kind: AbiIssueKind::PropertyOnNonLet,
                message: format!("widget `{widget_name}` not found in vx source"),
                span: None,
                severity: AbiSeverity::Error,
            }],
        };
    };

    let mut abi = ComponentAbi { widget_name: widget_name.to_string(), ..Default::default() };
    let mut emit_targets = Vec::new();

    for item in &widget.items {
        match item {
            StatementNode::Let(let_stmt) => extract_let_binding(source, let_stmt.as_ref(), &mut abi, &mut issues),
            StatementNode::ExprStmt(expr_stmt) => {
                collect_call_site(source, &expr_stmt.expr, std_range(&expr_stmt.span), &mut emit_targets, &mut abi, &mut issues);
            }
            StatementNode::Micro(micro) => {
                if has_attr(&micro.annotations, "event") {
                    extract_event_micro(source, micro.as_ref(), &mut abi, &mut issues);
                }
            }
            _ => {}
        }
    }

    validate_emit_targets(&abi, &emit_targets, &mut issues);
    warn_unused_events(&abi, &emit_targets, &mut issues);

    AbiExtractResult { abi, issues }
}

fn find_widget<'a>(root: &'a ValkyrieRoot, widget_name: &str) -> Option<&'a WidgetDeclaration> {
    root.items.iter().find_map(|item| match item {
        StatementNode::Widget(widget) if widget.name.name == widget_name => Some(widget.as_ref()),
        _ => None,
    })
}

fn extract_let_binding(source: &str, let_stmt: &Let, abi: &mut ComponentAbi, issues: &mut Vec<AbiIssue>) {
    let has_property = has_attr(&let_stmt.annotations, "property");
    let has_memoize = has_attr(&let_stmt.annotations, "memoize");
    let has_event = has_attr(&let_stmt.annotations, "event");

    if has_event {
        issues.push(error(AbiIssueKind::EventOnNonMicro, "`[event]` must annotate a `micro` declaration", Some(std_range(&let_stmt.span))));
    }
    if has_property && has_memoize {
        issues.push(error(
            AbiIssueKind::AttributeConflict,
            "`[property]` and `[memoize]` cannot be used together",
            Some(std_range(&let_stmt.span)),
        ));
    }
    if has_memoize && let_stmt.is_mutable {
        issues.push(error(AbiIssueKind::MemoizeOnMut, "`[memoize]` cannot be used on `let mut`", Some(std_range(&let_stmt.span))));
    }
    if has_property && let_stmt.is_mutable {
        issues.push(error(AbiIssueKind::AttributeConflict, "`[property]` cannot be used with `let mut`", Some(std_range(&let_stmt.span))));
    }

    let Some(name) = pattern_binding_name(&let_stmt.pattern)
    else {
        return;
    };

    let init_expr = Some(slice_span(source, std_range(&let_stmt.expr.span())));
    let type_hint = let_stmt.ty.as_ref().map(|ty| slice_span(source, type_expression_span(ty)));

    if has_property {
        abi.properties.push(AbiProperty {
            name: name.clone(),
            type_hint,
            required: false,
            default_expr: init_expr,
            span: std_range(&let_stmt.span),
        });
        return;
    }

    if has_memoize {
        if init_expr.is_none() {
            issues.push(error(AbiIssueKind::MemoizeOnMut, "`[memoize]` requires an initializer expression", Some(std_range(&let_stmt.span))));
            return;
        }
        let expr = init_expr.unwrap_or_default();
        if expr.len() < 12 {
            issues.push(warn(
                AbiIssueKind::MemoizeTrivial,
                format!("`[memoize]` on `{name}` may have little benefit for simple expressions"),
                Some(std_range(&let_stmt.span)),
            ));
        }
        abi.memoized.push(AbiMemo { name, expr, span: std_range(&let_stmt.span) });
        return;
    }

    if let_stmt.is_mutable {
        abi.states.push(AbiState { name, init_expr, span: std_range(&let_stmt.span) });
        return;
    }

    if let Some(expr) = init_expr {
        abi.derived.push(AbiDerived { name, expr, span: std_range(&let_stmt.span) });
    }
}

fn extract_event_micro(source: &str, micro: &MicroDeclaration, abi: &mut ComponentAbi, issues: &mut Vec<AbiIssue>) {
    if !is_empty_body(&micro.body) {
        issues.push(error(
            AbiIssueKind::EventNonEmptyBody,
            format!("`[event] micro {}` must have an empty body", micro.name.name),
            Some(std_range(&micro.span)),
        ));
    }
    if abi.events.iter().any(|event| event.name == micro.name.name) {
        issues.push(error(
            AbiIssueKind::AttributeConflict,
            format!("duplicate event `{}`", micro.name.name),
            Some(std_range(&micro.span)),
        ));
    }
    let params: Vec<AbiParam> = micro
        .params
        .iter()
        .map(|param| AbiParam {
            name: param.name.name.clone(),
            type_hint: param.ty.as_ref().map(|ty| slice_span(source, type_expression_span(ty))),
        })
        .collect();
    abi.events.push(AbiEvent { name: micro.name.name.clone(), params, span: std_range(&micro.span) });
}

fn collect_call_site(
    source: &str,
    expression: &TermExpression,
    span: Range<usize>,
    emit_targets: &mut Vec<(String, Range<usize>, usize)>,
    abi: &mut ComponentAbi,
    issues: &mut Vec<AbiIssue>,
) {
    let TermExpression::ApplyCall { callee, args, .. } = expression
    else {
        return;
    };
    let Some(callee) = callee_name(callee)
    else {
        return;
    };
    if callee == "emit" {
        if args.is_empty() {
            issues.push(error(AbiIssueKind::EmitInvalidTarget, "`emit` requires an event target", Some(span)));
            return;
        }
        if let Some(event_name) = arg_identifier(&args[0]) {
            emit_targets.push((event_name, span, args.len().saturating_sub(1)));
            return;
        }
        if arg_string_literal(&args[0]).is_some() {
            issues.push(warn(
                AbiIssueKind::LegacyStringEmit,
                "string `emit(\"name\", ...)` is deprecated; declare `[event] micro` and use `emit(name, ...)`",
                Some(span),
            ));
            return;
        }
        issues.push(error(AbiIssueKind::EmitInvalidTarget, "`emit` first argument must be an event symbol", Some(span)));
        return;
    }
    if callee == "effect" {
        let deps = args.iter().filter_map(arg_identifier).collect();
        abi.effects.push(AbiEffect { deps, span });
    }
}

fn validate_emit_targets(abi: &ComponentAbi, emit_targets: &[(String, Range<usize>, usize)], issues: &mut Vec<AbiIssue>) {
    for (event_name, span, arg_count) in emit_targets {
        let Some(event) = abi.event(event_name)
        else {
            issues.push(error(
                AbiIssueKind::EmitInvalidTarget,
                format!("`emit({event_name}, ...)` target is not a declared `[event] micro`"),
                Some(span.clone()),
            ));
            continue;
        };
        if *arg_count != event.params.len() {
            issues.push(error(
                AbiIssueKind::EmitArityMismatch,
                format!("`emit({event_name}, ...)` expects {} argument(s), got {arg_count}", event.params.len()),
                Some(span.clone()),
            ));
        }
    }
}

fn warn_unused_events(abi: &ComponentAbi, emit_targets: &[(String, Range<usize>, usize)], issues: &mut Vec<AbiIssue>) {
    for event in &abi.events {
        if !emit_targets.iter().any(|(name, _, _)| name == &event.name) {
            issues.push(warn(
                AbiIssueKind::EventNeverEmitted,
                format!("event `{}` is declared but never emitted", event.name),
                Some(event.span.clone()),
            ));
        }
    }
}

fn has_attr(annotations: &[Attribute], name: &str) -> bool {
    annotations.iter().any(|attr| attr.name.name == name)
}

fn is_empty_body(body: &Block) -> bool {
    body.statements.is_empty()
}

fn pattern_binding_name(pattern: &Pattern) -> Option<String> {
    match pattern {
        Pattern::Variable(variable) => Some(variable.name.name.clone()),
        _ => None,
    }
}

fn callee_name(expression: &TermExpression) -> Option<&str> {
    match expression {
        TermExpression::NamePath(path) => path.parts.last().map(|part| part.name.as_str()),
        _ => None,
    }
}

fn arg_identifier(expression: &TermExpression) -> Option<String> {
    match expression {
        TermExpression::NamePath(path) => path.parts.last().map(|part| part.name.clone()),
        _ => None,
    }
}

fn arg_string_literal(expression: &TermExpression) -> Option<String> {
    match expression {
        TermExpression::StringLiteral(literal) => Some(string_literal_text(literal)),
        _ => None,
    }
}

fn string_literal_text(literal: &StringLiteral) -> String {
    literal
        .segments
        .iter()
        .filter_map(|segment| match segment {
            StringSegment::Text(text) => Some(text.content.as_str()),
            StringSegment::Interpolation(_) => None,
        })
        .collect()
}

fn type_expression_span(expression: &TypeExpression) -> Range<usize> {
    match expression {
        TypeExpression::Binary(node) => std_range(&node.span),
        TypeExpression::Unary(node) => std_range(&node.span),
        TypeExpression::Generic(node) => std_range(&node.span),
        TypeExpression::Tuple(node) => std_range(&node.span),
        TypeExpression::Function(node) => std_range(&node.span),
        TypeExpression::Optional(node) => std_range(&node.span),
        TypeExpression::AssociatedType(node) => std_range(&node.span),
        TypeExpression::QualifiedAssociatedType(node) => std_range(&node.span),
        TypeExpression::Namepath(path) => std_range(&path.span),
    }
}

fn slice_span(source: &str, span: Range<usize>) -> String {
    source.get(span).unwrap_or("").trim().to_string()
}

fn error(kind: AbiIssueKind, message: impl Into<String>, span: Option<Range<usize>>) -> AbiIssue {
    AbiIssue { kind, message: message.into(), span, severity: AbiSeverity::Error }
}

fn warn(kind: AbiIssueKind, message: impl Into<String>, span: Option<Range<usize>>) -> AbiIssue {
    AbiIssue { kind, message: message.into(), span, severity: AbiSeverity::Warning }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_property_event_and_state_from_widget_script() {
        let source = r#"widget theme_switcher {
    [property] let theme = "";
    [property] let mode = "auto";
    let mut open = false;
    let resolved_mode = mode;
    [memoize] let visible_themes = filter(themes);
    [event] micro theme_change(theme: string) { }
    micro on_select(next_theme: string) {
        emit(theme_change, next_theme);
    }
}"#;
        let result = extract_component_abi_from_vx(source, "theme_switcher");
        assert!(result.issues.iter().all(|issue| issue.severity != AbiSeverity::Error), "{:?}", result.issues);
        assert_eq!(result.abi.properties.len(), 2);
        assert!(!result.abi.properties[0].required);
        assert!(!result.abi.properties[1].required);
        assert_eq!(result.abi.states.len(), 1);
        assert_eq!(result.abi.derived.len(), 1);
        assert_eq!(result.abi.memoized.len(), 1);
        assert_eq!(result.abi.events.len(), 1);
    }

    #[test]
    fn rejects_non_empty_event_body() {
        let source = r#"widget bad {
    [event] micro click() { return 0 }
}"#;
        let result = extract_component_abi_from_vx(source, "bad");
        assert!(result.issues.iter().any(|issue| issue.kind == AbiIssueKind::EventNonEmptyBody));
    }
}
