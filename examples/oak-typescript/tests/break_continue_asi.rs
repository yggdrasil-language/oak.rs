//! `break` and `continue` must not take a label across a line terminator:
//! ECMAScript forbids a newline between the keyword and its label. The old
//! `self.at(IdentifierName)` check skipped trivia (newlines included) and
//! swallowed the next statement's head as a label, derailing the parse.
//! The builder does not emit `Break`/`Continue` AST nodes yet, so these
//! tests gate on zero diagnostics.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage, ast::TypeScriptRoot};

fn assert_clean(source: &str) {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let output: oak_core::OakDiagnostics<TypeScriptRoot> = TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
}

#[test]
fn break_before_newline_statement_terminates() {
    assert_clean("function f() { while (x) { break\npos++ } }");
}

#[test]
fn continue_before_newline_statement_terminates() {
    assert_clean("function f() { while (x) { continue\npos++ } }");
}

#[test]
fn same_line_label_still_consumed() {
    assert_clean("function f() { for (;;) { break outer } }");
    assert_clean("function f() { for (;;) { continue outer } }");
}

#[test]
fn break_before_block_close() {
    assert_clean("function f() { while (x) { break\n} }");
}

#[test]
fn line_comment_then_newline_still_terminates() {
    assert_clean("function f() { while (x) { break // done\npos++ } }");
}

#[test]
fn break_before_keyword_statement() {
    assert_clean("function f() { while (x) { break\nreturn } }");
}

#[test]
fn same_line_semicolon_break_still_parses() {
    assert_clean("function f() { for (;;) { continue; x++ } }");
}
