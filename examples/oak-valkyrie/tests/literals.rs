use oak_core::{Builder, SourceText};
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage, ast::TermExpression};

#[test]
fn builder_preserves_numeric_literal_kind() {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let source = SourceText::new("micro main() { let a = 42; let b = 3.5 }");
    let mut session = oak_core::ParseSession::<ValkyrieLanguage>::default();
    let output = builder.build(&source, &[], &mut session);
    let root = output.result.expect("Oak 应构造包含数值字面量的 AST");
    let oak_valkyrie::ast::StatementNode::Micro(micro) = &root.items[0] else {
        panic!("缺少 micro 声明");
    };
    let [first, second] = micro.body.statements.as_slice() else {
        panic!("数值字面量测试需要两个 let 语句");
    };
    let oak_valkyrie::ast::Statement::Let(first) = first else { panic!("第一个语句不是 let") };
    let oak_valkyrie::ast::Statement::Let(second) = second else { panic!("第二个语句不是 let") };
    assert!(matches!(first.expr, TermExpression::IntegerLiteral { ref value, .. } if value == "42"));
    assert!(matches!(second.expr, TermExpression::FloatLiteral { ref value, .. } if value == "3.5"));
}
