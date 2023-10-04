use oak_notedown::diagnostic_set_from_notedown;

#[test]
fn stub_builder_exports_empty_unified_set() {
    let set = diagnostic_set_from_notedown("# Title\n\nBody.");
    assert!(set.diagnostics().is_empty());
}
