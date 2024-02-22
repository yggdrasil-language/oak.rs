use oak_awsl::formatter::{FormatOptions, format_source};

#[test]
fn formats_template_spacing() {
    let source = r#"<template><div class="box"></div></template>"#;
    let out = format_source(source, &FormatOptions::default()).expect("format");
    assert!(out.contains("<template>"));
    assert!(out.contains("class="));
}

#[test]
fn preserves_html_comment_gap() {
    let source = "<!-- note -->\n<template></template>\n";
    let out = format_source(source, &FormatOptions::default()).expect("format");
    assert!(out.contains("<!-- note -->"));
}

#[test]
fn idempotent_minimal_template() {
    let source = r#"
<template>
<div class="box"><span>{title}</span></div>
</template>
"#;
    let once = format_source(source, &FormatOptions::default()).expect("format once");
    let twice = format_source(&once, &FormatOptions::default()).expect("format twice");
    assert_eq!(once, twice);
}
