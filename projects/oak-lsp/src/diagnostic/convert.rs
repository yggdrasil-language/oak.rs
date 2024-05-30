use boost_diagnostic::{
    ByteRange, Diagnostic, DiagnosticLabel, DiagnosticLocation, DiagnosticSet, DiagnosticSeverity, Message, SourceRef,
    terminal::{SourceID, SourceProvider},
};
use oak_core::Arc;

use crate::types::{DiagnosticRelatedInformation, DiagnosticSeverity as ProtocolSeverity, StructuredDiagnostic, TextLocation, TextRange};

use super::{DiagnosticError, position::byte_range_to_text_range};

/// Resolve a structured [`SourceRef`] to a cached [`SourceID`].
pub trait SourceResolver {
    /// Resolve a source reference.
    fn resolve(&self, source: &SourceRef) -> Option<&SourceID>;
}

/// Convert every diagnostic in a set into protocol structured diagnostics.
pub fn structured_set_to_protocol(set: &DiagnosticSet, provider: &impl SourceProvider, resolver: &impl SourceResolver, uri_for: &impl Fn(&SourceID) -> Option<Arc<str>>) -> Result<Vec<StructuredDiagnostic>, DiagnosticError> {
    set.diagnostics().iter().map(|diagnostic| structured_to_protocol(diagnostic, provider, resolver, uri_for)).collect()
}

/// Convert a structured diagnostic into a protocol structured diagnostic.
pub fn structured_to_protocol(diagnostic: &Diagnostic, provider: &impl SourceProvider, resolver: &impl SourceResolver, uri_for: &impl Fn(&SourceID) -> Option<Arc<str>>) -> Result<StructuredDiagnostic, DiagnosticError> {
    let mut range = TextRange::empty();
    let mut has_range = false;
    let mut related_information = Vec::new();

    if let Some(primary) = diagnostic.primary() {
        match label_text_location(provider, resolver, uri_for, primary)? {
            Some(location) => {
                range = location.range;
                has_range = true;
            }
            None => related_information.push(message_only_related(location_summary(primary.location(), primary.message()))),
        }
    }

    for secondary in diagnostic.secondary() {
        match label_text_location(provider, resolver, uri_for, secondary)? {
            Some(location) => {
                related_information.push(DiagnosticRelatedInformation { location, message: render_message(secondary.message()) });
            }
            None => related_information.push(message_only_related(location_summary(secondary.location(), secondary.message()))),
        }
    }

    if !has_range {
        if let Some(first) = related_information.first() {
            range = first.location.range;
        }
    }

    Ok(StructuredDiagnostic {
        range,
        severity: Some(severity_to_protocol(diagnostic.severity())),
        code: Some(diagnostic.code().as_str().to_string()),
        source: Some(diagnostic.origin().to_string()),
        message: render_message(diagnostic.message()),
        related_information: if related_information.is_empty() { None } else { Some(related_information) },
    })
}

fn label_text_location(provider: &impl SourceProvider, resolver: &impl SourceResolver, uri_for: &impl Fn(&SourceID) -> Option<Arc<str>>, label: &DiagnosticLabel) -> Result<Option<TextLocation>, DiagnosticError> {
    match label.location() {
        DiagnosticLocation::Text { source, range } | DiagnosticLocation::Virtual { source, range } => {
            let file = resolver.resolve(source).ok_or_else(|| DiagnosticError::UnknownSource { wire_id: source.wire_id() })?;
            let uri = uri_for(file).ok_or_else(|| DiagnosticError::Provider(format!("missing URI for source {}", source.id())))?;
            let range = byte_range_to_text_range(provider, file, range.start(), range.end())?;
            Ok(Some(TextLocation { uri, range }))
        }
        _ => Ok(None),
    }
}

fn severity_to_protocol(severity: DiagnosticSeverity) -> ProtocolSeverity {
    match severity {
        DiagnosticSeverity::Bug | DiagnosticSeverity::Error => ProtocolSeverity::Error,
        DiagnosticSeverity::Warning => ProtocolSeverity::Warning,
        DiagnosticSeverity::Info => ProtocolSeverity::Information,
        DiagnosticSeverity::Hint => ProtocolSeverity::Hint,
    }
}

fn render_message(message: &Message) -> String {
    message.fallback().map(str::to_string).unwrap_or_else(|| message.key().to_string())
}

fn location_summary(location: &DiagnosticLocation, message: &Message) -> String {
    let prefix = match location {
        DiagnosticLocation::Binary { source, address_space, range, .. } => {
            format!("{}:{} [{}, {})", source.id(), address_space.id(), range.start(), range.end())
        }
        DiagnosticLocation::Member { container, member, range, precision, .. } => {
            let path = member.segments().iter().map(|segment| format!("{}/{}", segment.kind(), segment.name())).collect::<Vec<_>>().join(" -> ");
            let range = range.map(|range: ByteRange| format!(" [{}, {})", range.start(), range.end())).unwrap_or_default();
            format!("{}::{path}{range} ({precision:?})", container.id())
        }
        DiagnosticLocation::Object { source, object, .. } => {
            format!("{} {}/{}", source.id(), object.kind(), object.id())
        }
        DiagnosticLocation::Semantic { document, path, .. } => {
            format!("{}:{}", document.id(), path.as_str())
        }
        DiagnosticLocation::Text { source, range } | DiagnosticLocation::Virtual { source, range } => {
            format!("{} [{}, {})", source.id(), range.start(), range.end())
        }
    };
    format!("{prefix}: {}", render_message(message))
}

fn message_only_related(message: String) -> DiagnosticRelatedInformation {
    DiagnosticRelatedInformation { location: TextLocation { uri: Arc::from("file:///unknown"), range: TextRange::empty() }, message }
}
