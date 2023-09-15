use oak_core::{ParseSession, Parser, SourceText};
use oak_typescript::{TypeScriptLanguage, TypeScriptParser};
use std::time::{Duration, Instant};

#[test]
fn generated_event_shell_calls_parse_without_hang() {
    let source = SourceText::new(
        r#"export default class EventButton {
    constructor(props = {}) {
        if (typeof this.__vmzApplyProps === "function") this.__vmzApplyProps(props);
    }
}
EventButton.__vmzCreate = (function __vmzCreate(api) {
    var e0 = api.el("button");
    api.on(e0, "click", (() => this.armed = !this.armed));
    return e0;
});
"#,
    );
    let language = TypeScriptLanguage::default();
    let parser = TypeScriptParser::new(&language);
    let mut session = ParseSession::default();
    let start = Instant::now();
    let parsed = parser.parse(&source, &[], &mut session);
    assert!(start.elapsed() < Duration::from_millis(100), "parse took {:?}", start.elapsed());
    assert!(parsed.result.is_ok(), "diagnostics: {:?}", parsed.diagnostics);
}
