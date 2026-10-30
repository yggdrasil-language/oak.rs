//! Regex literal lexing: escaped `/` and `\]` inside character classes,
//! `\uXXXX` and `\p{...}` escapes, and unescaped `/` inside `[...]` must not
//! terminate the literal, while plain division keeps its binary meaning.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};

fn parse(source: &str) -> oak_core::OakDiagnostics<oak_typescript::ast::TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn assert_clean(source: &str) {
    let output = parse(source);
    assert!(output.result.is_ok(), "{source}: {:?}", output.result.err());
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
}

#[test]
fn regex_after_default_keeps_escaped_slash() {
    assert_clean("export default /[a\\/b]/;");
    assert_clean("const re = /[a\\/b]/;");
    assert_clean("const o = { re: /[a\\/b]/ };");
}

#[test]
fn regex_character_class_absorbs_slash_and_escapes() {
    assert_clean(r"const re = /[a/b]/;");
    assert_clean(r"const re = /[^\]\/\\]/g;");
    assert_clean(r"const re = /\p{P}[\u00B7\/]/u;");
    assert_clean(r"export default /\p{P}|\p{S}|\p{Cf}|[\u00AD\u034F\u061C\u180E\u200B-\u200F\u202A-\u202E\u2060-\u2064\u2066-\u206F\uFEFF\uFFF9-\uFFFB]|\p{Cn}|[\uD800-\uDBFF](?![\uDC00-\uDFFF])|(?<![\uD800-\uDBFF])[\uDC00-\uDFFF]/gu;");
}

#[test]
fn division_between_operands_stays_operator() {
    assert_clean("const x = a / b;");
    assert_clean("const x = (a + b) / c;");
    assert_clean("const x = arr.length / 2;");
}
