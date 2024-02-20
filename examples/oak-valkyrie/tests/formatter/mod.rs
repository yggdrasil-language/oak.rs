use oak_valkyrie::formatter::{FormatOptions, format_source};

#[test]
fn formats_micro_spacing() {
    let out = format_source("micro main(){let x=1}", &FormatOptions::default()).expect("format");
    assert!(out.contains("micro main()"));
    assert!(out.contains("let"));
}

#[test]
fn preserves_line_comment_gap() {
    let source = "# note\nmicro main() { }\n";
    let out = format_source(source, &FormatOptions::default()).expect("format");
    assert!(out.contains("# note"));
}

#[test]
fn idempotent_minimal_micro() {
    let source = "micro main() { let x = 1 }";
    let once = format_source(source, &FormatOptions::default()).expect("format once");
    let twice = format_source(&once, &FormatOptions::default()).expect("format twice");
    assert_eq!(once, twice);
}
