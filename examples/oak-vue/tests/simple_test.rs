use oak_core::{Builder, ParseSession, Parser, SourceText};
use oak_vue::{VueBuilder, VueLanguage, VueParser};

#[test]
fn test_simple_vue() {
    let source = SourceText::new("<template><div>{{ msg }}</div></template>");
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();

    let result = parser.parse(&source, &[], &mut session);
    assert!(result.diagnostics.is_empty(), "Parser should not return errors");

    println!("Parse successful!");
}

#[test]
fn dynamic_directive_args_parse_and_build_without_hang() {
    let source = SourceText::new(
        "<template><button :[attrName]=\"val\" @[eventName]=\"onEv\">x</button></template>",
    );
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
