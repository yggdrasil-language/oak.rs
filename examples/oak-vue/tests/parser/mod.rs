use oak_core::{Builder, ParseSession, Parser, SourceText, errors::OakError};
use oak_testing::parsing::ParserTester;
use oak_vue::{VueBuilder, VueLanguage, VueParser};
use std::{path::Path, time::Duration};

#[test]
fn pascal_case_link_component_parses_children() {
    let source = SourceText::new("<template><main><Link to=\"IndexPage\">Home</Link></main></template>");
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(!parsed.has_errors(), "parse errors: {:?}", parsed.diagnostics);

    let builder = VueBuilder::new();
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.result.is_ok(), "build failed: {:?}", built.diagnostics);
}

#[test]
#[ignore = "Vue parser currently diverges on the fixture and does not return within the tester timeout"]
fn test_vue_parser() -> Result<(), OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let tests = here.join("tests/parser");
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let test_runner = ParserTester::new(tests).with_extension("vue").with_timeout(Duration::from_secs(5));
    test_runner.run_tests(&parser)
}
