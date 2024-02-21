use oak_von::formatter::{FormatOptions, format_source};

#[test]
fn formats_object_spacing() {
    let out = format_source(r#"{x:1}"#, &FormatOptions::default()).expect("format");
    assert!(out.contains("{"));
    assert!(out.contains(":"));
}

#[test]
fn preserves_hash_comment_gap() {
    let source = "# note\n{ x: 1 }\n";
    let out = format_source(source, &FormatOptions::default()).expect("format");
    assert!(out.contains("# note"));
}

#[test]
fn idempotent_minimal_object() {
    let source = r#"{ "x": 1, "y": [true, null] }"#;
    let once = format_source(source, &FormatOptions::default()).expect("format once");
    let twice = format_source(&once, &FormatOptions::default()).expect("format twice");
    assert_eq!(once, twice);
}
