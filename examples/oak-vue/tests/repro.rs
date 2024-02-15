use oak_core::parser::ParseSession;
use oak_vue::{VueLanguage, VueParser};

#[test]
fn repro_vue_expressions() {
    let lang = VueLanguage::default();
    let parser = VueParser::new(&lang);
    for source in ["type", "readonly", "(() => toggle(item.id))", "((e) => note = e.target.value)", "({ id: \"sku-1\" })", "\"foo\" + (open ? \" is-opened\" : \"\")", "status || undefined"] {
        let mut session = ParseSession::<VueLanguage>::new(32);
        let result = parser.parse_expression_only(source, &mut session);
        if !result.diagnostics.is_empty() { println!("{source:?}: {:?}", result.diagnostics); }
        assert!(result.result.is_ok(), "{source}: {} diagnostics", result.diagnostics.len());
        assert!(result.diagnostics.is_empty(), "{source}: {} diagnostics", result.diagnostics.len());
    }
}
