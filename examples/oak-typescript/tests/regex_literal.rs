//! Regex literal lexing: escaped `/` and `\]` inside character classes,
//! `\uXXXX` and `\p{...}` escapes, and unescaped `/` inside `[...]` must not
//! terminate the literal, while plain division keeps its binary meaning.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, Statement},
};

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

/// The builder must keep regex literals as real expressions: a missing
/// `RegexLiteral` arm silently dropped the whole `export default /…/`
/// payload and initializer values (`value: None`).
#[test]
fn regex_literal_expressions_survive_into_ast() {
    let output = parse("export default /abc/;\nconst x = /re/g;\n");
    assert!(output.diagnostics.is_empty(), "{:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    let Statement::ExportDeclaration(export) = &root.statements[0]
    else {
        panic!("expected export declaration");
    };
    let inner = export.declaration.as_ref().expect("default payload");
    let Statement::ExpressionStatement(expr) = &**inner
    else {
        panic!("expected expression statement, got {inner:?}");
    };
    assert!(matches!(*expr.expression.kind, ExpressionKind::RegexLiteral(ref text) if text == "/abc/"), "{:?}", expr.expression.kind);
    let Statement::VariableDeclaration(var) = &root.statements[1]
    else {
        panic!("expected variable declaration");
    };
    let value = var.declarators[0].value.as_ref().expect("initializer kept");
    assert!(matches!(*value.kind, ExpressionKind::RegexLiteral(ref text) if text == "/re/g"), "{:?}", value.kind);
}
