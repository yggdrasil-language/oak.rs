use diagnostic::DiagnosticSeverity;
use oak_core::errors::{OakDiagnostics, OakError};
use oak_diagnostic::{diagnostic_set_from_output, from_oak_error, oak_wire_code};

#[test]
fn oak_error_maps_to_dotted_wire_code() {
    let error = OakError::unexpected_token("foo", 12, None);
    let diagnostic = from_oak_error(&error);
    assert_eq!(diagnostic.code().as_str(), "oak.syntax.unexpected_token");
    assert_eq!(diagnostic.severity(), DiagnosticSeverity::Error);
    assert!(diagnostic.primary().is_some());
}

#[test]
fn oak_diagnostics_collect_fatal_and_non_fatal() {
    let mut output = OakDiagnostics::success(());
    output.diagnostics.push(OakError::syntax_error("warn", 4, None));
    let set = diagnostic_set_from_output(&output);
    assert_eq!(set.diagnostics().len(), 1);

    let fatal: OakDiagnostics<()> = OakDiagnostics::error(OakError::unexpected_eof(0, None));
    let set = diagnostic_set_from_output(&fatal);
    assert_eq!(set.diagnostics().len(), 1);
    assert_eq!(
        oak_wire_code(fatal.result.as_ref().err().unwrap().kind()).as_str(),
        "oak.syntax.unexpected_eof"
    );
}
