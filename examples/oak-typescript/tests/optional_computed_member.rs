//! Optional computed member access (`obj?.[index]`) is a `MemberExpression`:
//! the index expression and the `?.` marker both survive, and the node is not
//! mistaken for an optional call (`?.(`).

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Expression, ExpressionKind, Statement, TypeScriptRoot},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn initializer(root: &TypeScriptRoot) -> &Expression {
    let Statement::VariableDeclaration(decl) = &root.statements[0]
    else {
        panic!("expected variable declaration: {:?}", root.statements)
    };
    decl.value().expect("initializer must exist")
}

#[test]
fn optional_computed_member_keeps_index_expression() {
    let source = "const x = a?.[i];";
    let root = build(source);
    let expr = initializer(&root);

    let ExpressionKind::MemberExpression { object, property, computed, optional } = &*expr.kind
    else {
        panic!("expected member expression: {:?}", expr.kind)
    };
    assert!(*computed, "`?.[` must stay a computed member access");
    assert!(*optional, "`?.[` must stay an optional access");
    assert!(matches!(&*object.kind, ExpressionKind::Identifier(name) if name == "a"));
    assert!(matches!(&*property.kind, ExpressionKind::Identifier(name) if name == "i"));
    assert_eq!(&source[expr.span.start..expr.span.end], "a?.[i]");
}

#[test]
fn optional_computed_member_chains_into_optional_property() {
    let source = "const x = a?.[0]?.b;";
    let root = build(source);
    let expr = initializer(&root);

    let ExpressionKind::MemberExpression { object, property, computed, optional } = &*expr.kind
    else {
        panic!("expected outer member expression: {:?}", expr.kind)
    };
    assert!(!*computed);
    assert!(*optional);
    assert!(matches!(&*property.kind, ExpressionKind::Identifier(name) if name == "b"));

    let ExpressionKind::MemberExpression { property: index, computed, optional, .. } = &*object.kind
    else {
        panic!("expected inner member expression: {:?}", object.kind)
    };
    assert!(*computed);
    assert!(*optional);
    assert!(matches!(&*index.kind, ExpressionKind::NumericLiteral(value) if *value == 0.0));
    assert_eq!(&source[expr.span.start..expr.span.end], "a?.[0]?.b");
}
