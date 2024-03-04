use oak_awsl::{AbiIssueKind, AbiSeverity, extract_component_abi_from_vx};

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
