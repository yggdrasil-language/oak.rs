use diagnostic::{DiagnosticSeverity, MessageArg};
use oak_core::errors::{OakDiagnostics, OakError};
use oak_diagnostic::{diagnostic_set_from_output, from_oak_error, oak_wire_code, OakDiagnosticsExt};

#[test]
fn oak_error_maps_to_dotted_wire_code() {
    let error = OakError::unexpected_token("foo", 12, None);
    let diagnostic = from_oak_error(&error);
    assert_eq!(diagnostic.code().as_str(), "oak.syntax.unexpected_token");
    assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
    assert!(diagnostic.primary().is_some());
    let args = diagnostic.message().args();
    assert!(args.iter().any(|(name, value)| name == "token" && matches!(value, MessageArg::Text(text) if text == "foo")));
    assert!(args.iter().any(|(name, value)| name == "offset" && matches!(value, MessageArg::U64(12))));
}

#[test]
fn oak_error_expected_token_carries_structured_args() {
    let error = OakError::expected_token("Semicolon", 8, None);
    let diagnostic = from_oak_error(&error);
    let args = diagnostic.message().args();
    assert!(args.iter().any(|(name, value)| name == "expected" && matches!(value, MessageArg::Text(text) if text == "Semicolon")));
    assert!(args.iter().any(|(name, value)| name == "offset" && matches!(value, MessageArg::U64(8))));
}

#[test]
fn oak_diagnostics_collect_fatal_and_non_fatal() {
    let mut output = OakDiagnostics::success(());
    output.diagnostics.push(OakError::syntax_error("warn", 4, None));
    let set = output.unified_set();
    assert_eq!(set.diagnostics().len(), 1);
    assert_eq!(diagnostic_set_from_output(&output).diagnostics().len(), 1);

    let fatal: OakDiagnostics<()> = OakDiagnostics::error(OakError::unexpected_eof(0, None));
    let set = fatal.unified_set();
    assert_eq!(set.diagnostics().len(), 1);
    assert_eq!(
        oak_wire_code(fatal.result.as_ref().err().unwrap().kind()).as_str(),
        "oak.syntax.unexpected_eof"
    );
}
