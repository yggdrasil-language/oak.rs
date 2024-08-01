//! `for` header forms: `for-in` (declared and bare), `for-of` and the classic
//! three-part loop must each build their own statement node without diagnostics.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn first_statement(source: &str) -> Statement {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    root.statements.first().expect("one statement expected").clone()
}

#[test]
fn for_in_with_const_declaration() {
    let stmt = first_statement("for (const k in o) x();");
    let Statement::ForInStatement(s) = stmt else {
        panic!("expected for-in, got {stmt:?}");
    };
    assert!(matches!(*s.left, Statement::VariableDeclaration(_)));
    let body = match &*s.body {
        Statement::ExpressionStatement(e) => e,
        other => panic!("expected expression body, got {other:?}"),
    };
    assert!(body.span.start < s.span.end);
}

#[test]
fn for_in_with_let_and_bare_forms() {
    assert!(matches!(first_statement("for (let i in o) { i; }"), Statement::ForInStatement(_)));
    assert!(matches!(first_statement("for (k in o) x();"), Statement::ForInStatement(_)));
}

#[test]
fn for_in_bare_left_keeps_member_chain() {
    let source = "for (a[i].b in o) x();";
    let stmt = first_statement(source);
    let Statement::ForInStatement(s) = stmt else {
        panic!("expected for-in, got {stmt:?}");
    };
    let Statement::ExpressionStatement(e) = &*s.left else {
        panic!("expected expression left, got {:?}", s.left);
    };
    assert_eq!(&source[e.span.start..e.span.end], "a[i].b");
}

#[test]
fn for_in_with_object_right_side() {
    assert!(matches!(first_statement("for (k in { a: 1 }) x();"), Statement::ForInStatement(_)));
}

#[test]
fn for_of_and_classic_for_still_parse() {
    assert!(matches!(first_statement("for (const v of arr) x();"), Statement::ForOfStatement(_)));
    assert!(matches!(first_statement("for (let i = 0; i < n; i++) x();"), Statement::ForStatement(_)));
    assert!(matches!(first_statement("for (;;) break;"), Statement::ForStatement(_)));
    assert!(matches!(first_statement("for (x = a ? b : c; ; ) ;"), Statement::ForStatement(_)));
}

#[test]
fn in_operator_inside_parens_stays_binary() {
    // A top-level `;` ends the header scan before the nested `in` matters.
    let stmt = first_statement("for (var ok = ('a' in b); ok; ) ;");
    let Statement::ForStatement(s) = stmt else {
        panic!("expected classic for, got {stmt:?}");
    };
    assert!(s.initializer.is_some());
}
