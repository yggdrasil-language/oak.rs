use oak_valkyrie::printer::{PrintOptions, PrintStyle, print_source};

#[test]
fn prints_empty_namespace_block() {
    let out = print_source("namespace demo {}", PrintStyle::Compact, &PrintOptions::default()).expect("print");
    assert_eq!(out, "namespace demo;");
}

#[test]
fn prints_minimal_micro() {
    let out = print_source("micro main(){let x=1}", PrintStyle::Compact, &PrintOptions::default()).expect("print");
    assert!(out.contains("micro main()"));
    assert!(out.contains("let x=1"));
}

#[test]
fn indented_micro_breaks_body_line() {
    let out = print_source("micro main(){let x=1}", PrintStyle::Indented, &PrintOptions::default()).expect("print");
    assert!(out.contains("{\n"));
    assert!(out.contains("let x=1"));
}
