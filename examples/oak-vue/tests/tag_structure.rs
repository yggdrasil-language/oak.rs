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
fn quoted_values_may_contain_angle_brackets() {
    // The unquoted-value terminator rules must not apply to a quoted value.
    assert_eq!(build(r#"<template><div a="<b>">x</div></template>"#), 0);
    assert_eq!(build(r#"<template><div a=">">x</div></template>"#), 0);
}

#[test]
fn adjacent_quoted_attributes_stay_separate() {
    assert_eq!(build(r#"<template><div a="1"b="2"></div></template>"#), 0);
}

#[test]
fn compound_equals_cannot_start_an_attribute() {
    // Vue lexes `==` as one token, so `a==b` is not an attribute with a value.
    assert_ne!(build("<template><div a==b></div></template>"), 0);
}

#[test]
fn unquoted_value_stops_at_a_quote() {
    assert_ne!(build(r#"<template><div a=b"c></div></template>"#), 0);
}

#[test]
fn a_start_tag_is_not_closed_by_a_nested_angle_bracket() {
    // `<div a="1"` followed by `<p>` must report an error instead of swallowing
    // the next tag as attributes.
    assert_ne!(build("<template><div a=\"1\"<p>x</p></template>"), 0);
}

#[test]
fn a_close_tag_without_an_open_element_is_an_error() {
    assert_ne!(build("</div>"), 0);
    assert_ne!(build("<template></div></template>"), 0);
}

#[test]
fn an_element_without_an_end_tag_is_an_error() {
    assert_ne!(build("<template><div>x"), 0);
    assert_ne!(build("<template><div>"), 0);
}
