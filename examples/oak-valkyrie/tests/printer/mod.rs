use oak_valkyrie::printer::{PrintOptions, print_source};

#[test]
fn prints_empty_namespace_block() {
    let options = PrintOptions::compact();
    let out = print_source("namespace demo {}", &options).expect("print");
    assert_eq!(out, "namespace demo;");
}

#[test]
fn prints_minimal_micro() {
    let options = PrintOptions::compact();
    let out = print_source("micro main(){let x=1}", &options).expect("print");
    assert!(out.contains("micro main()"));
    assert!(out.contains("let x=1"));
}

#[test]
fn render_document_preserves_semantics() {
    use oak_valkyrie::printer::{parse_source, render_document, to_document};

    let source = "micro main(){let x=1}";
    let root = parse_source(source).expect("parse");
    let doc = to_document(&root).expect("document");
    let out = render_document(&doc, &PrintOptions::indented());
    assert!(out.contains("micro main()"));
    assert!(out.contains("let x=1"));
}

#[test]
fn prints_return_and_call_micro() {
    let options = PrintOptions::compact();
    let source = "micro caller(value: i32) -> i32 { return foreign(value) }";
    let out = print_source(source, &options).expect("print");
    assert!(out.contains("micro caller(value: i32) -> i32"));
    assert!(out.contains("return foreign(value)"));
}

#[test]
fn prints_host_contract_attribute() {
    let options = PrintOptions::compact();
    let source = "[host_contract] micro foreign(value: i32) -> i32 { }";
    let out = print_source(source, &options).expect("print");
    assert!(out.contains("[host_contract]"));
    assert!(out.contains("micro foreign(value: i32) -> i32"));
}

#[test]
fn to_document_round_trip_matches_compact_print() {
    use oak_valkyrie::printer::{parse_source, print_root, to_document, render_document};

    let source = "micro main(){let x=1}";
    let root = parse_source(source).expect("parse");
    let doc = to_document(&root).expect("document");
    let compact = render_document(&doc, &PrintOptions::compact());
    let printed = print_root(&root, &PrintOptions::compact()).expect("print");
    assert_eq!(compact, printed);
}
