use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage, TypeScriptTokenType,
    ast::{ExpressionKind, Statement},
};

fn build(source: &str) -> oak_typescript::ast::TypeScriptRoot {
    let lang = TypeScriptLanguage { jsx: false, ..TypeScriptLanguage::standard() };
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.expect("root")
}

fn single_init(source: &str) -> oak_typescript::ast::Expression {
    let root = build(source);
    let Some(Statement::VariableDeclaration(decl)) = root.statements.first() else {
        panic!("{source}: expected variable declaration, got {:?}", root.statements);
    };
    match decl.declarators[0].value.clone() {
        Some(value) => value,
        None => panic!("{source}: no initializer in {:#?}", root.statements),
    }
}

/// `as`, `of`, `from`, `is`, `infer`, `keyof` and friends are contextual
/// keywords, not reserved words, so they are legal identifier references.
#[test]
fn contextual_keywords_parse_as_identifier_references() {
    for name in [
        "as", "of", "from", "is", "infer", "keyof", "unique", "asserts", "abstract",
        "declare", "namespace", "async", "get", "set", "type", "readonly", "constructor",
        "global", "require", "any", "boolean", "number", "object", "string", "symbol",
        "unknown", "never",
    ] {
        let source = format!("const value = {name};");
        let init = single_init(&source);
        assert!(
            matches!(&*init.kind, ExpressionKind::Identifier(actual) if actual == name),
            "{source}: expected identifier, got {:?}",
            init.kind
        );
    }
}

#[test]
fn contextual_keywords_parse_in_call_and_member_positions() {
    let root = build("register(as, of);\nobj.as;\nobj.of;");
    assert_eq!(root.statements.len(), 3, "{:?}", root.statements);
    let Some(Statement::ExpressionStatement(stmt)) = root.statements.first() else {
        panic!("expected expression statement");
    };
    let ExpressionKind::CallExpression { args, .. } = &*stmt.expression.kind else {
        panic!("expected call, got {:?}", stmt.expression.kind);
    };
    assert_eq!(args.len(), 2);
    assert!(matches!(&*args[0].kind, ExpressionKind::Identifier(name) if name == "as"));
    assert!(matches!(&*args[1].kind, ExpressionKind::Identifier(name) if name == "of"));
}

/// Reserved words stay out of the identifier-reference classification, so the
/// fix cannot widen into keywords that are not identifiers.
#[test]
fn reserved_word_tokens_are_not_identifier_references() {
    use TypeScriptTokenType::*;
    for kind in [Return, Class, Const, Typeof, New, Null, True, Function, Import, Extends] {
        assert!(!kind.is_contextual_keyword(), "{kind:?} must stay a reserved word");
    }
}
