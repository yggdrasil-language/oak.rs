use oak_xml::diagnostic_set_from_xml;

#[test]
fn valid_xml_exports_empty_unified_set() {
    let set = diagnostic_set_from_xml("<root/>");
    assert!(set.diagnostics().is_empty());
}

#[test]
fn malformed_xml_exports_unified_syntax_diagnostic() {
    let set = diagnostic_set_from_xml("<root><unclosed>");
    assert!(!set.diagnostics().is_empty());
    assert!(set.diagnostics().iter().all(|diagnostic| diagnostic.code().as_str().starts_with("oak.syntax.")));
}

#[test]
fn fatal_parse_error_maps_to_dotted_wire_code() {
    let set = diagnostic_set_from_xml("<");
    let diagnostic = set.diagnostics().first().expect("expected unified diagnostic");
    assert!(diagnostic.code().as_str().starts_with("oak.syntax."));
}
