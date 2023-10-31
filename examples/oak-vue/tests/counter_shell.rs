use oak_core::{Builder, ParseSession, Parser, SourceText};
use oak_vue::{VueBuilder, VueLanguage, VueParser};

#[test]
fn counter_directive_expressions_do_not_stall() {
    let cases = [r#"<button @input="((e) => note = e.target.value)">"#, r#"<button @click="(() => count++)">"#, r#"<select @change="((e) => plan = e.target.value)">"#];
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    for source in cases {
        let source = SourceText::new(source);
        let mut session = ParseSession::default();
        let parsed = parser.parse(&source, &[], &mut session);
        assert!(!parsed.has_errors(), "parse errors for {source:?}: {:?}", parsed.diagnostics);
    }
    let source = SourceText::new(
        r#"<template>
    <div data-testid="counter-root">
        <label>
            Note
            <input type="text" data-testid="counter-note" :value="note" @input="((e) => note = e.target.value)" />
        </label>
        <button type="button" aria-label="Increment counter" data-testid="counter-inc" @click="(() => count++)">
            count:
            {{ count }}
        </button>
        <label>
            Plan
            <select data-testid="counter-plan" :value="plan" @change="((e) => plan = e.target.value)">
                <option value>—</option>
                <option value="ops">Ops</option>
                <option value="dev">Dev</option>
            </select>
        </label>
    </div>
</template>"#,
    );
    let builder = VueBuilder::new();
    let mut build_session = ParseSession::default();
    let built = builder.build(&source, &[], &mut build_session);
    assert!(built.result.is_ok(), "full template build errors: {:?}", built.diagnostics);
}
