//! `for` headers with comma sequences: comma-separated init expressions,
//! multi-declarator `let` inits and comma-separated update expressions must
//! parse without diagnostics and keep every element in the AST.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, ForStatement, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn first_for(source: &str) -> ForStatement {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    let Some(Statement::ForStatement(stmt)) = root.statements.first().cloned() else {
        panic!("expected for statement in {source}");
    };
    stmt
}

#[test]
fn comma_init_sequence_keeps_both_assignments() {
    let stmt = first_for("for (i = 0, l = arr.length; i < l; i++) {}");
    let init = stmt.initializer.expect("init");
    let Statement::ExpressionStatement(expr) = &*init else {
        panic!("expected expression init, got {init:?}");
    };
    let ExpressionKind::SequenceExpression(expressions) = &*expr.expression.kind else {
        panic!("expected sequence init, got {:?}", expr.expression.kind);
    };
    assert_eq!(expressions.len(), 2);
    assert!(stmt.test.is_some());
    assert!(matches!(*stmt.incrementor.expect("incrementor").kind, ExpressionKind::UpdateExpression { .. }));
}

#[test]
fn let_multi_declarator_with_comma_updates() {
    let stmt = first_for("for (let i = 0, line = begin; line < end; line++, i++) {}");
    let init = stmt.initializer.expect("let init");
    let Statement::VariableDeclaration(decl) = &*init else {
        panic!("expected variable declaration init, got {init:?}");
    };
    assert_eq!(decl.declarators.len(), 2);
    let update = stmt.incrementor.expect("comma update");
    let ExpressionKind::SequenceExpression(expressions) = &*update.kind else {
        panic!("expected sequence update, got {:?}", update.kind);
    };
    assert_eq!(expressions.len(), 2);
}

#[test]
fn empty_init_with_comma_update_sequence() {
    let stmt = first_for("for (; offset < str.length; offset++, this.excess++) {}");
    assert!(stmt.initializer.is_none());
    assert!(stmt.test.is_some());
    let update = stmt.incrementor.expect("comma update");
    let ExpressionKind::SequenceExpression(expressions) = &*update.kind else {
        panic!("expected sequence update, got {:?}", update.kind);
    };
    assert_eq!(expressions.len(), 2);
}

#[test]
fn single_part_header_stays_plain_expression() {
    let stmt = first_for("for (i = 0; i < l; i++) {}");
    let init = stmt.initializer.expect("init");
    let Statement::ExpressionStatement(expr) = &*init else {
        panic!("expected expression init, got {init:?}");
    };
    assert!(matches!(*expr.expression.kind, ExpressionKind::AssignmentExpression { .. }));
    assert!(matches!(*stmt.incrementor.expect("update").kind, ExpressionKind::UpdateExpression { .. }));
}
