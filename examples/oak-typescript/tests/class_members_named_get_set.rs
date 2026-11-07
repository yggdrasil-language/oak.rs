//! Class members may be named `get` or `set`: the words are accessor prefixes
//! only when a member name follows (`get value() {}`), otherwise they are the
//! member name itself (`get(key) {}`, `set = 1`, `static get(key) {}`).
//! The class parser used to bump `get`/`set` unconditionally, mangling the
//! member and leaving the method body outside any function frame; scope facts
//! then reported a bogus top-level `return`.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, Statement, TypeScriptRoot},
    scope::{ScopeIncompleteReason, module_scope_facts},
};

fn parse(source: &str) -> oak_core::OakDiagnostics<TypeScriptRoot> {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache)
}

fn parsed_root(source: &str) -> TypeScriptRoot {
    let output = parse(source);
    assert!(output.diagnostics.is_empty(), "{source}: {:#?}", output.diagnostics);
    output.result.expect("source should parse")
}

fn class_members(source: &str) -> Vec<ClassMember> {
    let root = parsed_root(source);
    let Some(Statement::ClassDeclaration(declaration)) = root.statements.first().cloned()
    else {
        panic!("expected class declaration in {source}");
    };
    declaration.body
}

#[test]
fn method_named_get_keeps_function_frame() {
    let source = "class C {\n  get(key) {\n    if (cache.has(key)) { return 1; }\n    return 2;\n  }\n}";
    let members = class_members(source);
    let [ClassMember::Method { name, modifiers, .. }] = members.as_slice()
    else {
        panic!("expected one method, got {members:?}")
    };
    assert_eq!(name, "get");
    assert!(modifiers.is_empty(), "plain method must not carry accessor modifiers: {modifiers:?}");
    let reasons = module_scope_facts(&parsed_root(source)).incomplete_reasons;
    assert!(!reasons.contains(&ScopeIncompleteReason::TopLevelReturn), "{reasons:?}");
}

#[test]
fn method_named_set_keeps_function_frame() {
    let source = "class C {\n  set(key) {\n    if (cache.has(key)) { return 1; }\n  }\n}";
    let members = class_members(source);
    let [ClassMember::Method { name, modifiers, .. }] = members.as_slice()
    else {
        panic!("expected one method, got {members:?}")
    };
    assert_eq!(name, "set");
    assert!(modifiers.is_empty(), "plain method must not carry accessor modifiers: {modifiers:?}");
    let reasons = module_scope_facts(&parsed_root(source)).incomplete_reasons;
    assert!(!reasons.contains(&ScopeIncompleteReason::TopLevelReturn), "{reasons:?}");
}

#[test]
fn static_method_named_get_keeps_function_frame() {
    let source = "class C {\n  static get(key) {\n    return key;\n  }\n}";
    let members = class_members(source);
    let [ClassMember::Method { name, modifiers, .. }] = members.as_slice()
    else {
        panic!("expected one method, got {members:?}")
    };
    assert_eq!(name, "get");
    assert!(modifiers.iter().any(|m| m.kind == oak_typescript::ast::ModifierKind::Static));
    assert!(!modifiers.iter().any(|m| m.kind == oak_typescript::ast::ModifierKind::Get));
}

#[test]
fn field_named_get_stays_property() {
    let members = class_members("class C {\n  get = 1;\n}");
    let [ClassMember::Property { name, .. }] = members.as_slice()
    else {
        panic!("expected one property, got {members:?}")
    };
    assert_eq!(name, "get");
}

#[test]
fn real_getter_and_setter_still_detected() {
    let members = class_members("class C {\n  get value() { return 1; }\n  set value(v) {}\n}");
    let [ClassMember::Method { name: get_name, modifiers: get_modifiers, .. }, ClassMember::Method { name: set_name, modifiers: set_modifiers, .. }] = members.as_slice()
    else {
        panic!("expected getter/setter pair, got {members:?}")
    };
    assert_eq!(get_name, "value");
    assert!(get_modifiers.iter().any(|m| m.kind == oak_typescript::ast::ModifierKind::Get));
    assert_eq!(set_name, "value");
    assert!(set_modifiers.iter().any(|m| m.kind == oak_typescript::ast::ModifierKind::Set));
}
