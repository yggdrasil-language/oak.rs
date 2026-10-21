use std::collections::BTreeMap;

use boost_diagnostic::{
    ByteRange, Diagnostic, DiagnosticCode, DiagnosticLabel, DiagnosticLocation, DiagnosticOrigin, DiagnosticSeverity, LabelRole, Message,
    SourceRef, terminal::SourceID,
};
use lsp_types::{Position, Url};
use oak_lsp::diagnostic::{SourceCache, SourceResolver, byte_index_to_position, position_to_byte_index, structured_to_lsp};

const UNICODE: &str = "åä t𐐀b";

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

#[test]
fn unicode_get_byte_index() {
    let mut files = SourceCache::default();
    let file_id = files.load_text(UNICODE, "unicode");

    let result = position_to_byte_index(&files, &file_id, &Position { line: 0, character: 3 });
    assert_eq!(result.unwrap(), 5);

    let result = position_to_byte_index(&files, &file_id, &Position { line: 0, character: 6 });
    assert_eq!(result.unwrap(), 10);
}

#[test]
fn unicode_get_position() {
    let mut files = SourceCache::default();
    let file_id = files.load_text(UNICODE, "unicode");
    let file_id2 = files.load_text(format!("\n{UNICODE}"), "unicode2");

    let result = byte_index_to_position(&files, &file_id, 5);
    assert_eq!(result.unwrap(), Position { line: 0, character: 3 });

    let result = byte_index_to_position(&files, &file_id, 10);
    assert_eq!(result.unwrap(), Position { line: 0, character: 6 });

    let result = byte_index_to_position(&files, &file_id2, 11);
    assert_eq!(result.unwrap(), Position { line: 1, character: 6 });
}

#[test]
fn structured_text_diagnostic_to_lsp() {
    let mut cache = SourceCache::default();
    let source_ref = SourceRef::new("oak", "sample.tao").unwrap();
    let file_id = cache.load_text("let value = 1", "sample.tao");
    let resolver = TestResolver::new(&source_ref, file_id);

    let diagnostic = Diagnostic::new(
        DiagnosticCode::new("oak.syntax.unexpected-token"),
        DiagnosticSeverity::Error,
        DiagnosticOrigin::new("oak", "xml"),
        Message::new("oak.syntax.unexpected-token").with_fallback("unexpected token"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Text { source: source_ref, range: ByteRange::new(4, 9).unwrap() },
        Message::new("label.token").with_fallback("token"),
        LabelRole::Primary,
    ));

    let lsp = structured_to_lsp(&diagnostic, &cache, &resolver, &|_| Some(Url::parse("file:///sample.tao").unwrap())).unwrap();

    assert_eq!(lsp.message, "unexpected token");
    assert_eq!(lsp.severity, Some(lsp_types::DiagnosticSeverity::ERROR));
    assert_eq!(lsp.range.start.line, 0);
}

#[test]
fn structured_member_diagnostic_uses_related_information() {
    use boost_diagnostic::{MappingPrecision, MemberPath, MemberSegment};

    let cache = SourceCache::default();
    let resolver = TestResolver::new(&SourceRef::new("acorn", "docx.zip").unwrap(), SourceID::default());
    let diagnostic = Diagnostic::new(
        DiagnosticCode::new("acorn.container.need-range"),
        DiagnosticSeverity::Warning,
        DiagnosticOrigin::new("acorn", "zip"),
        Message::new("acorn.container.need-range").with_fallback("need range"),
    )
    .with_primary(DiagnosticLabel::new(
        DiagnosticLocation::Member {
            container: SourceRef::new("acorn", "docx.zip").unwrap(),
            member: MemberPath::new(vec![MemberSegment::new("zip", "word/document.xml")]),
            range: None,
            precision: MappingPrecision::Container,
        },
        Message::new("label.member").with_fallback("member"),
        LabelRole::Primary,
    ));

    let lsp = structured_to_lsp(&diagnostic, &cache, &resolver, &|_| None).unwrap();
    let related = lsp.related_information.expect("member location should become related info");
    assert!(related[0].message.contains("docx.zip"));
}
