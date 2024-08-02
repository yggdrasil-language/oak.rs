//! Tagged template literals: `tag`raw`` and `tag`a${x}b`` must build a
//! `TaggedTemplateExpression` with the tag and template as separate spans,
//! including inside ternaries and after member/call chains.

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

fn first_statement(source: &str) -> Statement {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    root.statements.first().expect("one statement expected").clone()
}

fn expression_kind(stmt: &Statement) -> &ExpressionKind {
    let Statement::ExpressionStatement(expr) = stmt else {
        panic!("expected expression statement, got {stmt:?}");
    };
    &expr.expression.kind
}

#[test]
fn tagged_template_builds_node_with_spans() {
    let source = "sql`SELECT * FROM t`;";
    let stmt = first_statement(source);
    let ExpressionKind::TaggedTemplateExpression { tag, template } = expression_kind(&stmt) else {
        panic!("expected tagged template, got {:?}", expression_kind(&stmt));
    };
    assert_eq!(&source[tag.span.start..tag.span.end], "sql");
    assert_eq!(&source[template.span.start..template.span.end], "`SELECT * FROM t`");
}

#[test]
fn tagged_template_inside_ternary() {
    let source = "negate ? r`\\B` : r`\\b`;";
    let stmt = first_statement(source);
    let ExpressionKind::ConditionalExpression { alternate, consequent, .. } = expression_kind(&stmt) else {
        panic!("expected conditional, got {:?}", expression_kind(&stmt));
    };
    assert!(matches!(*consequent.kind, ExpressionKind::TaggedTemplateExpression { .. }));
    assert!(matches!(*alternate.kind, ExpressionKind::TaggedTemplateExpression { .. }));
}

#[test]
fn tagged_template_with_interpolation_and_member_tag() {
    assert!(matches!(
        expression_kind(&first_statement("a.b`x${c}d`;")),
        ExpressionKind::TaggedTemplateExpression { .. }
    ));
    assert!(matches!(
        expression_kind(&first_statement("f()``;")),
        ExpressionKind::TaggedTemplateExpression { .. }
    ));
    assert!(matches!(
        expression_kind(&first_statement("css`${v}px`;")),
        ExpressionKind::TaggedTemplateExpression { .. }
    ));
}
