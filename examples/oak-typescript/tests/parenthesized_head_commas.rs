//! Parenthesized statement heads accept comma sequences because the
//! ECMAScript `Expression` production used by `if`, `while`, `do…while`,
//! `switch` and the `for` test slot includes `SequenceExpression`. The comma
//! must fold into a `SequenceExpression` instead of colliding with the
//! closing punctuation.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn first_stmt(source: &str) -> Statement {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    root.statements.first().cloned().unwrap_or_else(|| panic!("expected a statement in {source}"))
}

fn assert_sequence(kind: &ExpressionKind, expected: usize, label: &str) {
    let ExpressionKind::SequenceExpression(expressions) = kind else {
        panic!("expected sequence for {label}, got {kind:?}");
    };
    assert_eq!(expressions.len(), expected, "element count for {label}");
}

#[test]
fn switch_head_comma_sequence() {
    let stmt = first_stmt("switch(t.nextIndex++,u.type){case \"A\":break}");
    let Statement::SwitchStatement(switch) = stmt else {
        panic!("expected switch, got {stmt:?}");
    };
    assert_sequence(&switch.discriminant.kind, 2, "switch discriminant");
    assert_eq!(switch.cases.len(), 1);
}

#[test]
fn if_head_comma_sequence() {
    let stmt = first_stmt("if (a, b) x()");
    let Statement::IfStatement(if_stmt) = stmt else {
        panic!("expected if, got {stmt:?}");
    };
    assert_sequence(&if_stmt.test.kind, 2, "if test");
}

#[test]
fn while_head_comma_sequence() {
    let stmt = first_stmt("while (i++, i < n) {}");
    let Statement::WhileStatement(while_stmt) = stmt else {
        panic!("expected while, got {stmt:?}");
    };
    assert_sequence(&while_stmt.test.kind, 2, "while test");
}

#[test]
fn do_while_head_comma_sequence() {
    let stmt = first_stmt("do {} while (a, b);");
    let Statement::DoWhileStatement(do_stmt) = stmt else {
        panic!("expected do…while, got {stmt:?}");
    };
    assert_sequence(&do_stmt.test.kind, 2, "do…while test");
}

#[test]
fn for_test_slot_comma_sequence() {
    let stmt = first_stmt("for (; a, b; ) {}");
    let Statement::ForStatement(for_stmt) = stmt else {
        panic!("expected for, got {stmt:?}");
    };
    assert!(for_stmt.initializer.is_none());
    assert_sequence(&for_stmt.test.expect("test slot").kind, 2, "for test");
}

#[test]
fn three_part_head_sequence() {
    let stmt = first_stmt("if (a(), b(), c()) x()");
    let Statement::IfStatement(if_stmt) = stmt else {
        panic!("expected if, got {stmt:?}");
    };
    assert_sequence(&if_stmt.test.kind, 3, "three-part if test");
}

#[test]
fn grouped_sequence_inside_head_sequence() {
    let stmt = first_stmt("if ((a, b), c) x()");
    let Statement::IfStatement(if_stmt) = stmt else {
        panic!("expected if, got {stmt:?}");
    };
    assert_sequence(&if_stmt.test.kind, 2, "nested grouped if test");
}

#[test]
fn single_expression_heads_stay_plain() {
    let stmt = first_stmt("if (a) x()");
    let Statement::IfStatement(if_stmt) = stmt else {
        panic!("expected if, got {stmt:?}");
    };
    let ExpressionKind::Identifier(name) = &*if_stmt.test.kind else {
        panic!("expected identifier test, got {:?}", if_stmt.test.kind);
    };
    assert_eq!(name, "a");
}
