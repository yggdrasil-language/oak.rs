use oak_core::{Builder, SourceText};
use oak_valkyrie::{
    ValkyrieBuilder, ValkyrieLanguage,
    ast::{StatementNode, TermExpression},
};

#[test]
fn system_attributes_and_method_arguments_are_preserved() {
    let language = ValkyrieLanguage::ecs_language();
    let builder = ValkyrieBuilder::new(&language);
    let source = SourceText::new(
        r#"@phase("Update")
        system MovementSystem {
            @reads(Position, true, 42, factory("nested"))
            micro execute(world: World) { return 1 }
            micro finish() { return 2 }
        }
        system OtherSystem { micro execute() {} }"#,
    );
    let mut cache = oak_core::ParseSession::<ValkyrieLanguage>::default();
    let output = builder.build(&source, &[], &mut cache);
    let root = output.result.expect("system should build");
    assert_eq!(root.items.len(), 2);
    let StatementNode::System(system) = &root.items[0]
    else {
        panic!("expected system")
    };
    assert_eq!(system.name.name, "MovementSystem");
    assert_eq!(system.annotations.len(), 1);
    assert_eq!(system.annotations[0].name.name, "phase");
    assert!(matches!(&system.annotations[0].args[0], TermExpression::StringLiteral(literal) if literal.quote_count == 1));
    assert_eq!(system.methods.len(), 2);
    assert_eq!(system.methods[0].name.name, "execute");
    assert_eq!(system.methods[0].params.len(), 1);
    assert!(system.methods[0].body.is_some());
    let arguments = &system.methods[0].annotations[0].args;
    assert_eq!(arguments.len(), 4);
    assert!(matches!(&arguments[0], TermExpression::NamePath(path) if path.parts[0].name == "Position"));
    assert!(matches!(&arguments[1], TermExpression::Bool { value: true, .. }));
    assert!(matches!(&arguments[2], TermExpression::StringLiteral(literal) if literal.quote_count == 0));
    assert!(matches!(&arguments[3], TermExpression::ApplyCall { args, .. } if args.len() == 1));
    assert!(system.methods[1].annotations.is_empty());
    let StatementNode::System(other) = &root.items[1]
    else {
        panic!("expected system")
    };
    assert!(other.annotations.is_empty());
}
