//! A method return annotation is a real type, not a brace-blind scan: an
//! inline object type (`m(): { ok: boolean } { … }`) opens with `{`, and the
//! class parser used to mistake that brace for the method body. The remaining
//! members then fell out of the class body and an `async` follower surfaced as
//! a bogus top-level `expected Function` error.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, Statement, TypeScriptRoot},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn class_members(source: &str) -> Vec<ClassMember> {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    let root = output.result.expect("source should parse");
    let Some(Statement::ClassDeclaration(declaration)) = root.statements.first().cloned() else {
        panic!("expected class declaration in {source}");
    };
    declaration.body
}

fn member_names(members: &[ClassMember]) -> Vec<String> {
    members
        .iter()
        .map(|member| match member {
            ClassMember::Method { name, .. } | ClassMember::Property { name, .. } => name.clone(),
            other => panic!("unexpected member {other:?}"),
        })
        .collect()
}

#[test]
fn inline_object_return_annotation_keeps_async_follower() {
    let members = class_members("class C {\n  m(): { ok: boolean } {\n    return { ok: true };\n  }\n  async n() {}\n}");
    assert_eq!(member_names(&members), ["m", "n"]);
}

#[test]
fn multiline_inline_object_return_annotation() {
    let source = "class C {\n  m(): {\n    ok: boolean;\n    items: string[];\n  } {\n    return { ok: true, items: [] };\n  }\n  n() {}\n}";
    let members = class_members(source);
    assert_eq!(member_names(&members), ["m", "n"]);
}

#[test]
fn nested_generic_return_annotation_wrapping_object() {
    let members = class_members("class C {\n  m(): Promise<{ ok: boolean }> {\n    return null as never;\n  }\n  async n() {}\n}");
    assert_eq!(member_names(&members), ["m", "n"]);
}

#[test]
fn inline_object_return_annotation_before_getter() {
    let members = class_members("class C {\n  m(): { ok: boolean } {\n    return { ok: true };\n  }\n  get value() { return 1; }\n}");
    assert_eq!(member_names(&members), ["m", "value"]);
}

#[test]
fn inline_object_return_annotation_before_static_member() {
    let members = class_members("class C {\n  m(): { ok: boolean } {\n    return { ok: true };\n  }\n  static create() { return new C(); }\n}");
    assert_eq!(member_names(&members), ["m", "create"]);
}

#[test]
fn plain_return_annotations_still_parse() {
    let members = class_members("class C {\n  m(): void {}\n  n(): Promise<void> {}\n  o(): string | number { return 1; }\n}");
    assert_eq!(member_names(&members), ["m", "n", "o"]);
}

#[test]
fn overload_signature_without_body_still_parses() {
    let members = class_members("class C {\n  m(x: string): { ok: boolean };\n  m(x: number): { ok: number };\n  m(x: unknown): { ok: unknown } {\n    return { ok: x };\n  }\n}");
    assert_eq!(member_names(&members), ["m", "m", "m"]);
}
