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
    var config = { branches: [{ cond: function() { return this.armed; }, deps: [] }, { deps: [] }] };
    (function() {
        function patch() {
            try { var raw = this.armed ? "ON" : "OFF"; } catch {}
            for (var index = 0; index < config.branches.length; index++) {
                var branch = config.branches[index];
                if (!branch.cond) break;
                try { if (branch.cond.call(this)) break; } catch {}
            }
        }
        api.trackPatch(this, ["armed"], patch, 2);
    }).call(this);
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
    assert!(parsed.diagnostics.is_empty(), "diagnostics: {:?}", parsed.diagnostics);
}
