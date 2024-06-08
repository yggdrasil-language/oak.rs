use std::collections::BTreeMap;

use oak_core::Arc;

use boost_diagnostic::{ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, DiagnosticSet, DiagnosticSeverity, LabelRole, Message, SourceRef, terminal::SourceID};
use oak_lsp::diagnostic::{SourceCache, SourceResolver, structured_set_to_protocol, structured_to_protocol};

struct TestResolver(BTreeMap<String, SourceID>);

impl TestResolver {
    fn new(source: &SourceRef, id: SourceID) -> Self {
        let mut map = BTreeMap::new();
        map.insert(source.wire_id(), id);
        Self(map)
    }
}

impl SourceResolver for TestResolver {
    fn resolve(&self, source: &SourceRef) -> Option<&SourceID> {
        self.0.get(&source.wire_id())
    }
}

fn sample_text_diagnostic(source_ref: SourceRef) -> Diagnostic {
    Diagnostic::new(DiagnosticCode::new("oak.syntax.unexpected-token"), DiagnosticSeverity::Error, DiagnosticOrigin::new("oak", "xml"), Message::new("oak.syntax.unexpected-token").with_fallback("unexpected token")).with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Text { source: source_ref, range: ByteRange::new(4, 9).unwrap() },
        Message::new("label.token").with_fallback("token"),
        LabelRole::Primary,
    ))
}

#[test]
fn protocol_text_diagnostic_matches_golden_json() {
    let mut cache = SourceCache::default();
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();
    let file_id = cache.load_text("let value = 1", "sample.tao");
    let resolver = TestResolver::new(&source_ref, file_id);

    let protocol = structured_to_protocol(&sample_text_diagnostic(source_ref), &cache, &resolver, &|_| Some(Arc::from("file:///sample.tao"))).unwrap();

    let actual = serde_json::to_string_pretty(&protocol).expect("serialize protocol diagnostic");
    let expected = include_str!("golden/lsp-text.json");
    assert_eq!(actual.trim().replace("\r\n", "\n"), expected.trim().replace("\r\n", "\n"));
}

#[test]
fn protocol_set_conversion_preserves_order() {
    let mut cache = SourceCache::default();
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();
    let file_id = cache.load_text("let value = 1", "sample.tao");
    let resolver = TestResolver::new(&source_ref, file_id);

    let mut set = DiagnosticSet::new();
    set.push(sample_text_diagnostic(source_ref.clone()));
    set.push(sample_text_diagnostic(source_ref));

    let items = structured_set_to_protocol(&set, &cache, &resolver, &|_| Some(Arc::from("file:///sample.tao"))).unwrap();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0].message, items[1].message);
}

#[test]
#[ignore = "run manually to refresh protocol golden fixtures"]
fn write_protocol_golden_fixture() {
    use std::{fs, path::PathBuf};

    let mut cache = SourceCache::default();
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();
    let file_id = cache.load_text("let value = 1", "sample.tao");
    let resolver = TestResolver::new(&source_ref, file_id);

    let protocol = structured_to_protocol(&sample_text_diagnostic(source_ref), &cache, &resolver, &|_| Some(Arc::from("file:///sample.tao"))).unwrap();

    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/diagnostic/golden/lsp-text.json");
    fs::create_dir_all(path.parent().unwrap()).expect("create golden dir");
    fs::write(path, serde_json::to_string_pretty(&protocol).expect("serialize protocol diagnostic") + "\n").expect("write protocol golden");
}
