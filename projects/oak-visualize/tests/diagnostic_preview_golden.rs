use boost_diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, DiagnosticSeverity, LabelRole, Message,
    SourceRef,
    terminal::{SourceCache, SourceRegistry},
};
use oak_visualize::diagnostic_preview::structured_to_svg;

fn sample_text_diagnostic(source_ref: SourceRef) -> Diagnostic {
    Diagnostic::new(
        DiagnosticCode::new("svg.preview.sample"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("svg", "preview"),
        Message::new("svg.preview.sample").with_fallback("sample diagnostic"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Text { source: source_ref, range: ByteRange::new(0, 2).unwrap() },
        Message::new("svg.preview.label").with_fallback("fn"),
        LabelRole::Primary,
    ))
}

#[test]
fn svg_text_diagnostic_matches_golden() {
    let mut cache = SourceCache::default();
    let source = cache.load_text("fn main() {\n}\n", "main.rs");
    let source_ref = SourceRef::new("svg", "main.rs").unwrap();

    let mut registry = SourceRegistry::new();
    registry.register(&source_ref, source);

    let actual = structured_to_svg(&sample_text_diagnostic(source_ref), &cache, &registry).unwrap();
    let expected = include_str!("diagnostic_preview/golden/preview-text.svg");
    assert_eq!(actual, expected);
}

#[test]
#[ignore = "run manually to refresh SVG golden fixtures"]
fn write_svg_golden_fixture() {
    use std::{fs, path::PathBuf};

    let mut cache = SourceCache::default();
    let source = cache.load_text("fn main() {\n}\n", "main.rs");
    let source_ref = SourceRef::new("svg", "main.rs").unwrap();

    let mut registry = SourceRegistry::new();
    registry.register(&source_ref, source);

    let svg = structured_to_svg(&sample_text_diagnostic(source_ref), &cache, &registry).unwrap();
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/diagnostic_preview/golden/preview-text.svg");
    fs::create_dir_all(path.parent().unwrap()).expect("create golden dir");
    fs::write(path, svg).expect("write svg golden");
}
