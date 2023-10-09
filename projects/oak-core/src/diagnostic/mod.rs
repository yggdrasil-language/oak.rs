//! Adapters from Oak parser errors to the shared `diagnostic` contract.

pub use diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticAction, DiagnosticEnvelope,
    DiagnosticLabel as UnifiedDiagnosticLabel, DiagnosticLocation as UnifiedDiagnosticLocation,
    DiagnosticOrigin as UnifiedDiagnosticOrigin, DiagnosticSet,
    DiagnosticSeverity as UnifiedSeverity, DiagnosticSink, Message as UnifiedMessage, SCHEMA_VERSION,
};
use diagnostic::{
    ByteRange, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin,
    DiagnosticSeverity, LabelRole, Message, MessageArg, SourceRef,
};

use crate::errors::{OakDiagnostics, OakError, OakErrorKind};

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
    let id = match source_id {
        Some(id) => format!("source-{id}"),
        None => "anonymous".to_string(),
    };
    SourceRef::new("oak", id).unwrap_or_else(|_| {
        SourceRef::new("oak", "anonymous").expect("oak fallback source identity")
    })
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

fn message_for_kind(kind: &OakErrorKind) -> Message {
    let key = kind.key();
    let fallback = kind.to_string();
    let mut message = Message::new(key).with_fallback(fallback);
    if let Some(offset) = kind.source_offset() {
        message = message.with_arg("offset", MessageArg::U64(offset as u64));
    }
    match kind {
        OakErrorKind::IoError { source_id, .. } => {
            if let Some(id) = source_id {
                message = message.with_arg("source_id", MessageArg::U64(*id as u64));
            }
        }
        OakErrorKind::SyntaxError { message: text, .. } => {
            message = message.with_arg("message", MessageArg::Text(text.clone()));
        }
        OakErrorKind::UnexpectedCharacter { character, .. } => {
            message = message.with_arg("character", MessageArg::Text(character.to_string()));
        }
        OakErrorKind::UnexpectedToken { token, .. } => {
            message = message.with_arg("token", MessageArg::Text(token.clone()));
        }
        OakErrorKind::ExpectedToken { expected, .. } => {
            message = message.with_arg("expected", MessageArg::Text(expected.clone()));
        }
        OakErrorKind::ExpectedName { name_kind, .. } => {
            message = message.with_arg("name_kind", MessageArg::Text(name_kind.clone()));
        }
        OakErrorKind::CustomError { message: text }
        | OakErrorKind::InvalidTheme { message: text }
        | OakErrorKind::FormatError { message: text }
        | OakErrorKind::SemanticError { message: text }
        | OakErrorKind::ProtocolError { message: text }
        | OakErrorKind::SerdeError { message: text }
        | OakErrorKind::DeserializeError { message: text }
        | OakErrorKind::XmlError { message: text }
        | OakErrorKind::ZipError { message: text }
        | OakErrorKind::ParseError { message: text }
        | OakErrorKind::InternalError { message: text } => {
            message = message.with_arg("message", MessageArg::Text(text.clone()));
        }
        OakErrorKind::UnsupportedFormat { format } => {
            message = message.with_arg("format", MessageArg::Text(format.clone()));
        }
        OakErrorKind::ColorParseError { color } => {
            message = message.with_arg("color", MessageArg::Text(color.clone()));
        }
        OakErrorKind::TestFailure { path, expected, actual } => {
            message = message
                .with_arg("path", MessageArg::Text(path.display().to_string()))
                .with_arg("expected", MessageArg::Text(expected.clone()))
                .with_arg("actual", MessageArg::Text(actual.clone()));
        }
        OakErrorKind::TestRegenerated { path } => {
            message = message.with_arg("path", MessageArg::Text(path.display().to_string()));
        }
        OakErrorKind::UnexpectedEof { .. } | OakErrorKind::TrailingCommaNotAllowed { .. } => {}
    }
    message
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
    let mut diagnostic = Diagnostic::new(
        wire,
        severity,
        DiagnosticOrigin::new("oak", "parser"),
        message_for_kind(kind),
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

/// Project parser output into unified diagnostics and emit each record through [`logger`].
#[cfg(feature = "logger")]
pub fn emit_unified_output<T>(output: &OakDiagnostics<T>) {
    diagnostic::emit_diagnostic_set(&output.unified_set());
}

/// Project Oak parser errors and emit each unified diagnostic through [`logger`].
#[cfg(feature = "logger")]
pub fn emit_oak_errors(errors: &[OakError]) {
    let mut set = DiagnosticSet::new();
    for error in errors {
        set.push(from_oak_error(error));
    }
    diagnostic::emit_diagnostic_set(&set);
}
