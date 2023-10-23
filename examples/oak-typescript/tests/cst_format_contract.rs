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

#[test]
fn preserves_trailing_comment_and_block_comment_in_statement() {
    let input = "const x = 1 /* mid */ // end";
    let out = format_source(input, &CstFormatOptions::default()).expect("format");
    assert_eq!(out, input);
}

#[test]
fn preserves_asi_sensitive_continuation_line() {
    let input = "const total = base\n+ extra";
    let out = format_source(input, &CstFormatOptions::default()).expect("format");
    assert_eq!(out, input);
}

#[test]
fn preserves_block_comment_between_statements() {
    let input = "const a = 1\n/* between */\nconst b = 2";
    let out = format_source(input, &CstFormatOptions::default()).expect("format");
    assert_eq!(out, "const a = 1\n/* between */\nconst b = 2");
}

#[test]
fn formats_scoped_import_without_comments() {
    let input = "import  {  foo }  from 'pkg'";
    let out = format_source(input, &CstFormatOptions::default()).expect("format");
    assert_eq!(out, "import { foo } from 'pkg';");
}

#[test]
fn preserves_decorated_const_statement() {
    let input = "@Component()\nconst  x=1";
    let out = format_source(input, &CstFormatOptions::default()).expect("format");
    assert_eq!(out, input);
}
