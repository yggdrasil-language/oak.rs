//! Interpolation expressions must accept the full expression grammar, including
//! the ternary operator: `{{ a ? b : c }}` used to fail with
//! `expected InterpolationEnd` at the `?`.

use oak_core::{Builder, ParseSession, SourceText};
use oak_vue::VueBuilder;

fn diagnostics(src: &str) -> Vec<String> {
    let source = SourceText::new(src);
    let mut session = ParseSession::default();
    let built = Builder::build(&VueBuilder::new(), &source, &[], &mut session);
    let mut out: Vec<String> = built.diagnostics.iter().map(|d| format!("{d:?}")).collect();
    if built.result.is_err() {
        out.push("no parse result".to_string());
    }
    out
}

fn assert_ok(src: &str) {
    let errors = diagnostics(src);
    assert!(errors.is_empty(), "{src}: {errors:#?}");
}

#[test]
fn interpolation_accepts_ternary() {
    assert_ok("<template><span>{{ a ? b : c }}</span></template>");
}

#[test]
fn interpolation_accepts_ternary_with_member_and_equality() {
    assert_ok("<template><span>{{ a.id === current ? 'y' : '' }}</span></template>");
}

#[test]
fn interpolation_accepts_nested_ternary() {
    assert_ok("<template><span>{{ a ? b ? 1 : 2 : 3 }}</span></template>");
}

#[test]
fn interpolation_accepts_ternary_inside_call() {
    assert_ok("<template><span>{{ pick(a ? b : c) }}</span></template>");
}
