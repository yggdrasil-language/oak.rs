//! Shared CST format contract fixtures (capability matrix row `oak-typescript::cst_format`).

use oak_typescript::cst_format::{CstFormatOptions, format_source};

#[test]
fn preserves_leading_line_comment_and_normalizes_const() {
    let input = "// keep\nconst  x=1";
    let out = format_source(input, &CstFormatOptions::default()).expect("format");
    assert_eq!(out, "// keep\nconst x = 1");
    let again = format_source(&out, &CstFormatOptions::default()).expect("twice");
    assert_eq!(out, again);
}

#[test]
fn rejects_unsupported_top_level_class() {
    assert!(format_source("class Foo {}", &CstFormatOptions::default()).is_err());
}
