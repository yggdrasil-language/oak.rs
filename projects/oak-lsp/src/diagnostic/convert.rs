use boost_diagnostic::{
    ByteRange, Diagnostic, DiagnosticLabel, DiagnosticLocation, DiagnosticSet, DiagnosticSeverity, Message, SourceRef,
    terminal::{SourceID, SourceProvider},
};
use lsp_types::{
    Diagnostic as LspDiagnostic, DiagnosticRelatedInformation, DiagnosticSeverity as LspSeverity, Location, NumberOrString, Position, Range,
    Url,
};

use super::{DiagnosticError, position::byte_range_to_lsp_range};

/// Resolve a structured [`SourceRef`] to a cached [`SourceID`].
pub trait SourceResolver {
    /// Resolve a source reference.
    fn resolve(&self, source: &SourceRef) -> Option<&SourceID>;
}

/// Convert every diagnostic in a set into LSP diagnostics.
pub fn structured_set_to_lsp(
    set: &DiagnosticSet,
    provider: &impl SourceProvider,
    resolver: &impl SourceResolver,
    uri_for: &impl Fn(&SourceID) -> Option<Url>,
) -> Result<Vec<LspDiagnostic>, DiagnosticError> {
    set.diagnostics().iter().map(|diagnostic| structured_to_lsp(diagnostic, provider, resolver, uri_for)).collect()
}

/// Convert a structured diagnostic into an LSP diagnostic.
pub fn structured_to_lsp(
    diagnostic: &Diagnostic,
    provider: &impl SourceProvider,
    resolver: &impl SourceResolver,
    uri_for: &impl Fn(&SourceID) -> Option<Url>,
) -> Result<LspDiagnostic, DiagnosticError> {
    let mut range = empty_range();
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
            Some(location) => related_information.push(DiagnosticRelatedInformation { location, message: render_message(secondary.message()) }),
            None => related_information.push(message_only_related(location_summary(secondary.location(), secondary.message()))),
        }
    }

    if !has_range {
        if let Some(first) = related_information.first() {
            range = first.location.range;
        }
    }

    Ok(LspDiagnostic {
        range,
        severity: Some(severity_to_lsp(diagnostic.severity())),
        code: Some(NumberOrString::String(diagnostic.code().as_str().to_string())),
        code_description: None,
        source: Some(diagnostic.origin().to_string()),
        message: render_message(diagnostic.message()),
        related_information: if related_information.is_empty() { None } else { Some(related_information) },
        tags: None,
        data: None,
    })
}

fn label_text_location(
    provider: &impl SourceProvider,
    resolver: &impl SourceResolver,
    uri_for: &impl Fn(&SourceID) -> Option<Url>,
    label: &DiagnosticLabel,
) -> Result<Option<Location>, DiagnosticError> {
    match label.location() {
        DiagnosticLocation::Text { source, range } | DiagnosticLocation::Virtual { source, range } => {
            let file = resolver.resolve(source).ok_or_else(|| DiagnosticError::UnknownSource { wire_id: source.wire_id() })?;
            let uri = uri_for(file).ok_or_else(|| DiagnosticError::Provider(format!("missing URI for source {}", source.id())))?;
            let range = byte_range_to_lsp_range(provider, file, range.start(), range.end())?;
            Ok(Some(Location { uri, range }))
        }
        _ => Ok(None),
    }
}

fn severity_to_lsp(severity: DiagnosticSeverity) -> LspSeverity {
    match severity {
        DiagnosticSeverity::Bug | DiagnosticSeverity::Error => LspSeverity::ERROR,
        DiagnosticSeverity::Warning => LspSeverity::WARNING,
        DiagnosticSeverity::Info => LspSeverity::INFORMATION,
        DiagnosticSeverity::Hint => LspSeverity::HINT,
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
            let path =
                member.segments().iter().map(|segment| format!("{}/{}", segment.kind(), segment.name())).collect::<Vec<_>>().join(" -> ");
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

fn empty_range() -> Range {
    Range::new(Position::new(0, 0), Position::new(0, 0))
}

fn message_only_related(message: String) -> DiagnosticRelatedInformation {
    DiagnosticRelatedInformation {
        location: Location { uri: Url::parse("file:///unknown").expect("static unknown URI"), range: empty_range() },
        message,
    }
}
