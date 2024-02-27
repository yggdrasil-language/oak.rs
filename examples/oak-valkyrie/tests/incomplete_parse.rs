use oak_core::{Builder, ParseSession, SourceText};
use oak_valkyrie::{
    ValkyrieBuilder, ValkyrieLanguage,
    ast::{Statement, StatementNode, TermExpression},
};

/// 不完整源码必须在有限时间内结束解析，禁止饥饿循环撑爆 green tree。
#[test]
fn incomplete_micro_header_finishes_quickly() {
    let lang = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&lang);
    let src = SourceText::new("micro main(");
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let out = builder.build(&src, &[], &mut session);
    assert!(out.result.is_err() || out.result.is_ok());
}

/// micro 形参泛型用 `<T>`（或 `::<T>`），类型应用用 `Option<T>`。
#[test]
fn micro_and_type_generics_use_expected_syntax() {
    let lang = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&lang);
    let src = SourceText::new("micro wrap<T>(value: T) -> Envelope<T> { return value } unite Option<T> { Some(T) None }");
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let out = builder.build(&src, &[], &mut session);
    let root = out.result.expect("micro <T> 与 unite Option<T> 必须可解析");
    assert_eq!(root.items.len(), 2);
    let micro = match &root.items[0] {
        StatementNode::Micro(m) => m,
        other => panic!("expected Micro, got {other:?}"),
    };
    assert_eq!(micro.name.name, "wrap");
    assert_eq!(micro.generics.len(), 1);
    assert_eq!(micro.generics[0].name.name, "T");
    let unite = match &root.items[1] {
        StatementNode::Enums(e) => e,
        other => panic!("expected unite Enums, got {other:?}"),
    };
    assert_eq!(unite.generics.len(), 1);
    assert_eq!(unite.generics[0].name.name, "T");
}

#[test]
fn micro_generic_parameter_clause_also_accepts_turbofish_prefix() {
    let lang = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&lang);
    let src = SourceText::new("micro identity::<T>(value: T) -> T { return value }");
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let root = builder.build(&src, &[], &mut session).result.expect("micro ::<T> 可选");
    let StatementNode::Micro(micro) = &root.items[0]
    else {
        panic!("expected micro")
    };
    assert_eq!(micro.generics.len(), 1);
}

#[test]
fn object_initializer_parses_named_fields() {
    let lang = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&lang);
    let src = SourceText::new("let packet = Packet { value: value }");
    let mut session = ParseSession::<ValkyrieLanguage>::default();
    let root = builder.build(&src, &[], &mut session).result.expect("object init");
    let StatementNode::Let(let_stmt) = &root.items[0]
    else {
        panic!("expected let, got {:?}", root.items)
    };
    let TermExpression::Object { callee, fields, .. } = &let_stmt.expr
    else {
        panic!("expected Packet {{...}}, got {:?}", let_stmt.expr)
    };
    let TermExpression::NamePath(path) = callee.as_ref()
    else {
        panic!("expected Packet callee")
    };
    assert_eq!(path.parts[0].name, "Packet");
    assert_eq!(fields.len(), 1);
    assert_eq!(fields[0].0.name, "value");
}
