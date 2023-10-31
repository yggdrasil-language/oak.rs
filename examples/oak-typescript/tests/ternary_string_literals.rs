use oak_core::{Builder, ParseSession, SourceText};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ExpressionKind, Statement},
};

#[test]
fn conditional_string_literals_decode_without_trailing_trivia() {
    let source = SourceText::new(r#"error ? "true" : "false""#);
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = ParseSession::default();
    let built = Builder::build(&builder, &source, &[], &mut cache);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let root = built.result.expect("build");
    let expr = root
        .statements
        .iter()
        .find_map(|stmt| match stmt {
            Statement::ExpressionStatement(es) => Some(&es.expression),
            _ => None,
        })
        .expect("expression statement");
    match expr.kind.as_ref() {
        ExpressionKind::ConditionalExpression { consequent, alternate, .. } => {
            match consequent.kind.as_ref() {
                ExpressionKind::StringLiteral(s) => assert_eq!(s, "true", "consequent literal"),
                other => panic!("consequent: {other:?}"),
            }
            match alternate.kind.as_ref() {
                ExpressionKind::StringLiteral(s) => assert_eq!(s, "false", "alternate literal"),
                other => panic!("alternate: {other:?}"),
            }
        }
        other => panic!("root: {other:?}"),
    }
}
