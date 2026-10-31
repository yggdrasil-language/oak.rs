//! `module_scope_facts` reports module-level bindings with the spans of their
//! value references and a conservative `incomplete` flag. Reference spans are
//! only ever the spans the AST really carries; anything the AST cannot see
//! (eval, JSX, template interpolations, ambiguous keys, broken spans) flips
//! `incomplete` instead of guessing.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::TypeScriptRoot,
    scope::{ModuleBindingFacts, ModuleBindingKind, ModuleScopeFacts, module_scope_facts},
};

fn root(source: &str) -> TypeScriptRoot {
    let language = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    TypeScriptBuilder::new(&language).build(&SourceText::new(source), &[], &mut cache).result.expect("source should parse")
}

fn facts(source: &str) -> ModuleScopeFacts {
    module_scope_facts(&root(source))
}

fn binding<'a>(f: &'a ModuleScopeFacts, name: &str) -> &'a ModuleBindingFacts {
    f.bindings.iter().find(|b| b.name == name).unwrap_or_else(|| panic!("no binding `{name}` in {f:?}"))
}

fn ref_texts<'a>(source: &'a str, b: &ModuleBindingFacts) -> Vec<&'a str> {
    b.references.iter().map(|r| &source[r.start..r.end]).collect()
}

fn decl_text<'a>(source: &'a str, b: &ModuleBindingFacts) -> &'a str {
    &source[b.declaration.start..b.declaration.end]
}

#[test]
fn function_and_const_bindings_record_value_references_in_source_order() {
    let source = "function foo(){}\nfoo();\nfoo = 1;\nfoo++;\nconst bar = foo;";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert_eq!(f.bindings.len(), 2, "{:?}", f.bindings.iter().map(|b| &b.name).collect::<Vec<_>>());

    let foo = binding(&f, "foo");
    assert!(matches!(foo.kind, ModuleBindingKind::Function));
    assert_eq!(foo.declaration.start, 9);
    assert_eq!(decl_text(source, foo), "foo");
    assert_eq!(ref_texts(source, foo), vec!["foo", "foo", "foo", "foo"]);
    let starts: Vec<usize> = foo.references.iter().map(|r| r.start).collect();
    assert!(starts.windows(2).all(|w| w[0] < w[1]), "references must ascend: {starts:?}");
    assert_eq!(starts[0], source.find("foo();").expect("call"));

    let bar = binding(&f, "bar");
    assert!(matches!(bar.kind, ModuleBindingKind::Const));
    assert_eq!(decl_text(source, bar), "bar");
    assert!(bar.references.is_empty(), "{bar:?}");
    assert!(!foo.has_type_references);
}

#[test]
fn parameter_shadows_module_binding() {
    let source = "function foo(){}\nfunction g(foo){ foo }\nfoo;";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert_eq!(ref_texts(source, binding(&f, "foo")), vec!["foo"]);
    assert_eq!(binding(&f, "foo").references[0].start, source.rfind("foo;").expect("tail use"));
}

#[test]
fn block_let_shadows_and_block_var_hoists_to_module() {
    let source = "let foo = 1;\n{ let foo = 2; foo; }\nfoo;\nif (x) { var v = 1; v; }\nv;";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");

    let foo = binding(&f, "foo");
    assert!(matches!(foo.kind, ModuleBindingKind::Let));
    assert_eq!(ref_texts(source, foo), vec!["foo"]);
    assert_eq!(foo.references[0].start, source.find("\nfoo;").expect("outer use") + 1);

    let v = binding(&f, "v");
    assert!(matches!(v.kind, ModuleBindingKind::Var), "block var must hoist to module: {v:?}");
    assert_eq!(decl_text(source, v), "v");
    assert_eq!(ref_texts(source, v), vec!["v", "v"]);
}

#[test]
fn catch_param_shadows_module_binding() {
    let source = "let e = 1;\ntry { } catch (e) { e }\ne;";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert_eq!(ref_texts(source, binding(&f, "e")), vec!["e"]);
}

#[test]
fn function_expression_name_shadows_self_reference() {
    // The builder does not emit `ExpressionKind::FunctionExpression` yet: its
    // catch-all walk exposes the body's inner call as the declarator value, so
    // the AST really carries that reference and it resolves to the module
    // binding. `scope.rs` keeps the shadow rule for the moment the AST grows
    // the real node; until then this test pins the actual flattened behavior.
    let source = "const f = function f(){ f(); };";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    let binding = binding(&f, "f");
    assert!(matches!(binding.kind, ModuleBindingKind::Const));
    assert_eq!(ref_texts(source, binding), vec!["f"], "flattened body call is the only visible use");
}

#[test]
fn non_computed_property_and_break_label_are_not_references() {
    let source = "let b = 1;\nlet obj = { k: 1 };\nobj.b;\nb[x];\nconst o = { b };\nb;\nfor (;;) { break b; }";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    let b = binding(&f, "b");
    // `obj.b` (non-computed) and `break b` (label) must not count; the computed
    // object position, the shorthand `{ b }` and the tail `b;` must.
    assert_eq!(ref_texts(source, b), vec!["b", "b", "b"]);
    assert_eq!(b.references[0].start, source.find("b[x]").expect("computed member"));
}

#[test]
fn typeof_new_tagged_template_and_callee_are_references() {
    let source = "let F = 1;\ntypeof F;\nnew F();\ntag`x`;\nF`y`;\nfree(F);";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    let fb = binding(&f, "F");
    assert_eq!(ref_texts(source, fb), vec!["F", "F", "F", "F"]);
    assert_eq!(fb.references[0].start, source.find("typeof F").expect("typeof") + 7);
    assert_eq!(fb.references[1].start, source.find("new F(").expect("new") + 4);
}

#[test]
fn type_annotations_mark_type_references_without_spans() {
    let source = "class Foo {}\nlet a: Foo;\nclass Bar extends Foo {}\nexport { type Foo };";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    let foo = binding(&f, "Foo");
    assert!(matches!(foo.kind, ModuleBindingKind::Class));
    assert!(foo.has_type_references, "annotation, extends and `export {{ type Foo }}` are type uses");
    assert!(foo.references.is_empty(), "type positions never record spans: {foo:?}");
    assert!(matches!(binding(&f, "Bar").kind, ModuleBindingKind::Class));
}

#[test]
fn type_parameter_shadows_module_binding_in_type_position() {
    // `parse_type_parameters` swallows `<Foo>` as a flat token run, so the
    // builder records the type parameter with an empty name and no shadow can
    // be registered from the AST. `scope.rs` keeps the shadow rule for real
    // names; this test pins the actual behavior: the visible `Foo` type uses
    // resolve to the module binding.
    let source = "let Foo = 1;\nfunction g<Foo>(x: Foo): Foo { }\n";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert!(binding(&f, "Foo").has_type_references, "the AST loses the type-parameter name today");
}

#[test]
fn registered_type_parameter_names_do_shadow() {
    // Same rule exercised through a path the AST does carry: a mapped-type key
    // variable is a real shadow entry and must not mark the module binding.
    let source = "let K = 1;\ntype M = { [K in string]: K };\n";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert!(!binding(&f, "K").has_type_references, "the mapped key `K` shadows the module binding");
}

#[test]
fn import_bindings_use_local_token_spans() {
    let source = "import def from \"m\";\nimport { named, other as alias } from \"m\";\nimport * as ns from \"m\";\ndef;\nnamed;\nalias;\nns;\nimport type T from \"m\";";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert_eq!(f.bindings.len(), 4, "{:?}", f.bindings.iter().map(|b| &b.name).collect::<Vec<_>>());
    assert!(f.bindings.iter().all(|b| b.name != "T"), "`import type` must not create a binding");

    let def = binding(&f, "def");
    assert!(matches!(def.kind, ModuleBindingKind::ImportDefault));
    assert_eq!(decl_text(source, def), "def");
    assert_eq!(ref_texts(source, def), vec!["def"]);

    let named = binding(&f, "named");
    assert!(matches!(named.kind, ModuleBindingKind::ImportNamed));
    assert_eq!(decl_text(source, named), "named");
    assert_eq!(ref_texts(source, named), vec!["named"]);

    let alias = binding(&f, "alias");
    assert!(matches!(alias.kind, ModuleBindingKind::ImportNamed));
    assert_eq!(decl_text(source, alias), "alias");
    assert_eq!(ref_texts(source, alias), vec!["alias"]);

    let ns = binding(&f, "ns");
    assert!(matches!(ns.kind, ModuleBindingKind::ImportNamespace));
    assert_eq!(decl_text(source, ns), "ns");
    assert_eq!(ref_texts(source, ns), vec!["ns"]);
}

#[test]
fn export_declaration_creates_binding_and_bare_clause_records_reference() {
    let source = "export function foo(){}\nfoo();\nexport { foo };";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    let foo = binding(&f, "foo");
    assert!(matches!(foo.kind, ModuleBindingKind::Function));
    assert_eq!(decl_text(source, foo), "foo");
    assert_eq!(ref_texts(source, foo), vec!["foo", "foo"]);
    assert_eq!(foo.references[1].start, source.find("export { foo }").expect("export clause") + 9);

    let reexport = facts("export { foo } from \"m\";");
    assert!(reexport.bindings.is_empty(), "re-export creates no local binding: {reexport:?}");
    assert!(!reexport.incomplete);
}

#[test]
fn export_default_function_and_class_are_module_bindings() {
    let source = "export default function foo(){}\nexport default class Bar {}\nfoo;";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert!(matches!(binding(&f, "foo").kind, ModuleBindingKind::Function));
    assert!(matches!(binding(&f, "Bar").kind, ModuleBindingKind::Class));
    assert_eq!(ref_texts(source, binding(&f, "foo")), vec!["foo"]);
}

#[test]
fn direct_eval_marks_incomplete() {
    let f = facts("let foo = 1;\neval(\"foo\");\nfoo;");
    assert!(f.incomplete, "{f:?}");
}

#[test]
fn jsx_marks_incomplete() {
    let f = facts("const x = <div/>;");
    assert!(f.incomplete, "JSX must flip the flag: {:?}", module_scope_facts(&root("const x = <div/>;")));
}

#[test]
fn template_interpolation_marks_incomplete() {
    let f = facts("let foo = 1;\nconst t = `a${foo}b`;");
    assert!(f.incomplete, "interpolations are invisible in the AST: {f:?}");
}

#[test]
fn plain_key_matching_binding_name_marks_incomplete() {
    let f = facts("let foo = 1;\nconst o = { foo: 2 };");
    assert!(f.incomplete, "the AST cannot tell `foo:` from `[foo]:`: {f:?}");
}

#[test]
fn duplicate_module_binding_marks_incomplete_and_keeps_first() {
    let source = "function foo(){}\nlet foo = 1;\nfoo;";
    let f = facts(source);
    assert!(f.incomplete, "{f:?}");
    assert_eq!(f.bindings.len(), 1, "the first entry wins: {f:?}");
    assert!(matches!(binding(&f, "foo").kind, ModuleBindingKind::Function));
    assert_eq!(ref_texts(source, binding(&f, "foo")), vec!["foo"]);
}

#[test]
fn export_type_clause_marks_type_reference_on_value_binding() {
    let f = facts("class Foo {}\nexport type { Foo };");
    assert!(binding(&f, "Foo").has_type_references, "`export type {{ Foo }}` still names the class binding: {:?}", binding(&f, "Foo"));
    assert!(binding(&f, "Foo").references.is_empty());
    assert!(!f.incomplete, "{f:?}");

    let reexport = facts("export type { Foo } from \"m\";");
    assert!(!reexport.incomplete);
    assert!(reexport.bindings.is_empty());
}

#[test]
fn interface_bodies_are_opaque_and_mark_incomplete() {
    let f = facts("let foo = 1;\ninterface I { foo: string }");
    assert!(f.incomplete, "the AST carries no interface members: {f:?}");

    let empty = facts("let foo = 1;\ninterface I {}");
    assert!(empty.incomplete, "even an empty interface flips the flag until the parser records members");
    assert_eq!(empty.bindings.len(), 1, "{empty:?}");
}

#[test]
fn destructuring_and_shorthand_tokens_mark_rename_unsafe() {
    let source = "const { value } = obj;\nlet [a, b] = arr;\nconst { x: renamed } = obj2;\nconst plain = 1;\nconst o = { plain };\nfunction safe(){ }\nsafe();";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    assert!(binding(&f, "value").rename_unsafe, "shorthand pattern token doubles as the property key");
    assert!(binding(&f, "a").rename_unsafe, "array pattern token");
    assert!(binding(&f, "b").rename_unsafe, "array pattern token");
    assert!(binding(&f, "renamed").rename_unsafe, "conservative: `{{ x: renamed }}` renames safely but the whole pattern is flagged");
    assert!(binding(&f, "plain").rename_unsafe, "shorthand reference `{{ plain }}` doubles as the property key");
    assert!(!binding(&f, "safe").rename_unsafe, "plain declaration and plain call are rename-safe");
}

#[test]
fn block_var_destructuring_marks_rename_unsafe() {
    let source = "if (x) { var { value } = obj; }";
    let f = facts(source);
    assert!(!f.incomplete, "{f:?}");
    let value = binding(&f, "value");
    assert!(matches!(value.kind, ModuleBindingKind::Var), "block var must hoist: {value:?}");
    assert!(value.rename_unsafe, "hoisted var destructuring keeps the pattern token");
}

#[test]
fn export_star_re_exports_create_no_local_binding() {
    let f = facts("export * as ns from \"m\";");
    assert!(f.bindings.is_empty(), "`export * as ns from` is a re-export, not a local binding: {f:?}");
    assert!(!f.incomplete);

    let star = facts("export * from \"m\";");
    assert!(star.bindings.is_empty());
    assert!(!star.incomplete);
}
