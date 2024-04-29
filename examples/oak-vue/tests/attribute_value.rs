use oak_core::{Builder, ParseSession, SourceText};
use oak_vue::VueBuilder;

fn build(src: &str) -> usize {
    let source = SourceText::new(src);
    let mut session = ParseSession::default();
    let built = Builder::build(&VueBuilder::new(), &source, &[], &mut session);
    let total = built.diagnostics.len() + usize::from(built.result.is_err());
    for d in &built.diagnostics {
        println!("{d:?}");
    }
    total
}

#[test]
fn unquoted_attribute_value_does_not_swallow_the_closing_tag() {
    // Vue treats an unquoted plain-attribute value as static text.
    assert_eq!(build("<template><p title={a}>x</p></template>"), 0);
    assert_eq!(build("<template><p if={ok}>{label}</p></template>"), 0);
    assert_eq!(build("<template><p title={a} id={b}>x</p></template>"), 0);
}

#[test]
fn unquoted_attribute_value_stops_at_whitespace() {
    assert_eq!(build("<template><p title=a id=b>x</p></template>"), 0);
}
