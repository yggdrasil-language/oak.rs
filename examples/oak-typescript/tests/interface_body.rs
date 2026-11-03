use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, Statement, TypeAnnotation},
};

fn interface(source: &str) -> (Vec<ClassMember>, Vec<TypeAnnotation>) {
    let lang = TypeScriptLanguage { jsx: false, ..TypeScriptLanguage::standard() };
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    let root = result.result.expect("root");
    let Some(Statement::Interface(declaration)) = root.statements.first() else {
        panic!("{source}: expected an interface, got {:#?}", root.statements);
    };
    (declaration.body.clone(), declaration.extends.clone())
}

#[test]
fn interface_body_records_property_and_method_signatures() {
    let (members, _) = interface(
        "interface Props {\n  readonly id?: string\n  name: string\n  render(count: number): string\n}",
    );
    assert_eq!(members.len(), 3, "{members:#?}");
    assert!(matches!(members[0], ClassMember::Property { ref name, optional_marker: Some(_), .. } if name == "id"));
    assert!(matches!(members[1], ClassMember::Property { ref name, optional_marker: None, .. } if name == "name"));
    let ClassMember::Method { name, params, return_type, .. } = &members[2] else {
        panic!("expected a method signature, got {:?}", members[2]);
    };
    assert_eq!(name, "render");
    assert_eq!(params.len(), 1);
    assert!(matches!(return_type, Some(TypeAnnotation::Identifier(type_name)) if type_name == "string"));
}

#[test]
fn interface_heritage_records_extended_types() {
    let (_, extends) = interface("interface Props extends Base, Other<string> {}");
    assert_eq!(extends.len(), 2, "{extends:#?}");
}

#[test]
fn interface_body_records_index_call_and_construct_signatures() {
    let (members, _) = interface(
        "interface Callable {\n  [key: string]: number\n  (value: string): number\n  new (value: string): Callable\n}",
    );
    assert_eq!(members.len(), 3, "{members:#?}");
    let ClassMember::Method { return_type, .. } = &members[0] else {
        panic!("expected index signature as a method node, got {:?}", members[0]);
    };
    assert!(matches!(return_type, Some(TypeAnnotation::Identifier(name)) if name == "number"));
    let ClassMember::Method { params, return_type, .. } = &members[1] else {
        panic!("expected call signature, got {:?}", members[1]);
    };
    assert_eq!(params.len(), 1);
    assert!(matches!(return_type, Some(TypeAnnotation::Identifier(name)) if name == "number"));
    let ClassMember::Method { params, return_type, .. } = &members[2] else {
        panic!("expected construct signature, got {:?}", members[2]);
    };
    assert_eq!(params.len(), 1);
    assert!(matches!(return_type, Some(TypeAnnotation::Identifier(name)) if name == "Callable"));
}
