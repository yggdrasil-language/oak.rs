//! Class method bodies must lower `this` / update / private idents for VMZ FieldRw.

use oak_core::{Builder, ParseSession, SourceText};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{ClassMember, ExpressionKind, Statement},
};

fn build_root(src: &str) -> oak_typescript::TypeScriptRoot {
    let source = SourceText::new(src);
    let language = TypeScriptLanguage::default();
    let builder = TypeScriptBuilder::new(&language);
    let mut cache = ParseSession::default();
    Builder::build(&builder, &source, &[], &mut cache).result.expect("build ok")
}

fn class_methods(root: &oak_typescript::TypeScriptRoot) -> Vec<&ClassMember> {
    let mut out = Vec::new();
    for stmt in &root.statements {
        let class = match stmt {
            Statement::ExportDeclaration(exp) => match exp.declaration.as_deref() {
                Some(Statement::ClassDeclaration(c)) => Some(c),
                _ => None,
            },
            Statement::ClassDeclaration(c) => Some(c),
            _ => None,
        };
        if let Some(c) = class {
            for m in &c.body {
                if matches!(m, ClassMember::Method { .. }) {
                    out.push(m);
                }
            }
        }
    }
    out
}

#[test]
fn method_body_builds_this_member_update() {
    let root = build_root(
        r#"
export default class C {
  count = 0;
  bump() { this.count++; }
}
"#,
    );
    let methods = class_methods(&root);
    let ClassMember::Method { name, body, .. } = methods[0]
    else {
        panic!("expected method");
    };
    assert_eq!(name, "bump");
    assert_eq!(body.len(), 1, "body={body:?}");
    let Statement::ExpressionStatement(es) = &body[0]
    else {
        panic!("expected expr stmt, got {:?}", body[0]);
    };
    let ExpressionKind::UpdateExpression { argument, operator, .. } = es.expression.kind.as_ref()
    else {
        panic!("expected update, got {:?}", es.expression.kind);
    };
    assert_eq!(operator, "++");
    let ExpressionKind::MemberExpression { object, property, computed: false, .. } = argument.kind.as_ref()
    else {
        panic!("expected member, got {:?}", argument.kind);
    };
    assert!(matches!(object.kind.as_ref(), ExpressionKind::Identifier(n) if n == "this"));
    assert!(matches!(property.kind.as_ref(), ExpressionKind::Identifier(n) if n == "count"));
}

#[test]
fn method_body_builds_assignment_and_private_call() {
    let root = build_root(
        r#"
export default class Card {
  user = null;
  onClick() {
    this.user = 1;
    this.#load();
  }
  #load() { return this.user; }
}
"#,
    );
    let methods = class_methods(&root);
    let names: Vec<_> = methods
        .iter()
        .filter_map(|m| match m {
            ClassMember::Method { name, .. } => Some(name.as_str()),
            _ => None,
        })
        .collect();
    assert!(names.contains(&"onClick"), "names={names:?}");
    assert!(names.contains(&"#load"), "names={names:?}");

    let on_click = methods.iter().find(|m| matches!(m, ClassMember::Method { name, .. } if name == "onClick")).expect("onClick");
    let ClassMember::Method { body, .. } = on_click
    else {
        unreachable!()
    };
    assert!(body.len() >= 2, "body={body:?}");
}
