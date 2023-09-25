//! Adapters from Oak parser errors to the shared `diagnostic` contract.

use diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin,
    DiagnosticSet, DiagnosticSeverity, LabelRole, Message, SourceRef,
};
use oak_core::errors::{OakDiagnostics, OakError, OakErrorKind};

/// Convert an Oak i18n key (`error.*`) to a dotted wire code (`oak.syntax.*`).
pub fn oak_wire_code(kind: &OakErrorKind) -> DiagnosticCode {
    let key = kind.key();
    let suffix = key.strip_prefix("error.").unwrap_or(key);
    DiagnosticCode::new(format!("oak.syntax.{suffix}"))
}

/// Map Oak error kinds to unified diagnostic severity.
pub fn oak_severity(kind: &OakErrorKind) -> DiagnosticSeverity {
    match kind {
        OakErrorKind::TestRegenerated { .. } => DiagnosticSeverity::Info,
        OakErrorKind::InternalError { .. } | OakErrorKind::SerdeError { .. } => DiagnosticSeverity::Bug,
        _ => DiagnosticSeverity::Error,
    }
}

fn source_ref(source_id: Option<u32>) -> SourceRef {
    match source_id {
        Some(id) => SourceRef::new("oak", format!("source-{id}")),
        None => SourceRef::new("oak", "anonymous"),
    }
}

fn text_label(source_id: Option<u32>, offset: usize, span_len: usize) -> Option<DiagnosticLabel> {
    let end = offset.saturating_add(span_len.max(1));
    let range = ByteRange::new(offset as u64, end as u64).ok()?;
    let location = DiagnosticLocation::Text {
        source: source_ref(source_id),
        range,
    };
    Some(DiagnosticLabel::new(
        location,
        Message::new("oak.label.here").with_fallback("here"),
        LabelRole::Primary,
    ))
}

fn primary_label(kind: &OakErrorKind) -> Option<DiagnosticLabel> {
    match kind {
        OakErrorKind::SyntaxError { offset, source_id, .. }
        | OakErrorKind::UnexpectedCharacter { offset, source_id, .. }
        | OakErrorKind::UnexpectedToken { offset, source_id, .. }
        | OakErrorKind::UnexpectedEof { offset, source_id, .. }
        | OakErrorKind::ExpectedToken { offset, source_id, .. }
        | OakErrorKind::ExpectedName { offset, source_id, .. }
        | OakErrorKind::TrailingCommaNotAllowed { offset, source_id, .. } => {
            text_label(*source_id, *offset, 1)
        }
        _ => None,
    }
}

/// Convert one `OakError` into a unified diagnostic record.
pub fn from_oak_error(error: &OakError) -> Diagnostic {
    let kind = error.kind();
    let wire = oak_wire_code(kind);
    let severity = oak_severity(kind);
    let fallback = kind.to_string();
    let mut diagnostic = Diagnostic::new(
        wire,
        severity,
        DiagnosticOrigin::new("oak", "parser"),
        Message::new(kind.key()).with_fallback(fallback),
    );
    if let Some(label) = primary_label(kind) {
        diagnostic = diagnostic.with_primary(label);
    }
    diagnostic
}

/// Collect unified diagnostics from an Oak parse output.
pub fn diagnostic_set_from_output<T>(output: &OakDiagnostics<T>) -> DiagnosticSet {
    output.unified_set()
}

/// Extension methods for exporting Oak parser output to the shared diagnostic contract.
pub trait OakDiagnosticsExt<T> {
    /// Collect fatal and non-fatal parser errors into a [`DiagnosticSet`].
    fn unified_set(&self) -> DiagnosticSet;
}

impl<T> OakDiagnosticsExt<T> for OakDiagnostics<T> {
    fn unified_set(&self) -> DiagnosticSet {
        let mut set = DiagnosticSet::new();
        if let Err(error) = &self.result {
            set.push(from_oak_error(error));
        }
        for error in &self.diagnostics {
            set.push(from_oak_error(error));
        }
        set
    }
}
