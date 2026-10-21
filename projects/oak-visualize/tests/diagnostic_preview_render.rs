use boost_diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, DiagnosticSeverity, LabelRole, Message,
    SourceRef,
    terminal::{SourceCache, SourceRegistry},
};
use oak_visualize::diagnostic_preview::structured_to_svg;

#[test]
fn structured_text_diagnostic_renders_svg() {
    let mut cache = SourceCache::default();
    let source = cache.load_text("fn main() {\n}\n", "main.rs");
    let source_ref = SourceRef::new("svg", "main.rs").unwrap();

    let mut registry = SourceRegistry::new();
    registry.register(&source_ref, source);

    let diagnostic = Diagnostic::new(
        DiagnosticCode::new("svg.preview.sample"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("svg", "preview"),
        Message::new("svg.preview.sample").with_fallback("sample diagnostic"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Text { source: source_ref, range: ByteRange::new(0, 2).unwrap() },
        Message::new("svg.preview.label").with_fallback("fn"),
        LabelRole::Primary,
    ));

    let svg = structured_to_svg(&diagnostic, &cache, &registry).unwrap();
    assert!(svg.contains("<svg"));
    assert!(svg.contains("sample diagnostic"));
}
