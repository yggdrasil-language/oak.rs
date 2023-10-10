use oak_core::{Builder, ParseSession, SourceText};
use oak_typescript::{TypeScriptBuilder, TypeScriptLanguage};
use oak_typescript::ast::{ClassMember, ExpressionKind, Statement, Visibility};

#[test]
fn class_keyword_members_preserve_names_and_initializers() {
    let source = SourceText::new("export default class Button { public type: string = 'button'; public default = false; public delete = null; variant = 'primary'; }");
    let language = TypeScriptLanguage::default();
    let mut session = ParseSession::default();
    let built = TypeScriptBuilder::new(&language).build(&source, &[], &mut session);
    assert!(built.diagnostics.is_empty(), "{:?}", built.diagnostics);
    let root = built.result.expect("class AST");
    let Statement::ExportDeclaration(export) = &root.statements[0] else { panic!("export") };
    let Some(Statement::ClassDeclaration(class)) = export.declaration.as_deref() else { panic!("class") };
    assert_eq!(class.body.len(), 4);
    for (member, expected) in class.body.iter().zip(["type", "default", "delete", "variant"]) {
        let ClassMember::Property { name, initializer, .. } = member else { panic!("property") };
        assert_eq!(name, expected);
        assert!(initializer.is_some(), "{name}");
    }
    let ClassMember::Property { visibility, initializer, .. } = &class.body[0] else { panic!("type property") };
    assert!(matches!(visibility, Some(Visibility::Public)));
    assert!(matches!(initializer.as_ref().unwrap().kind.as_ref(), ExpressionKind::StringLiteral(value) if value == "button"));
}
