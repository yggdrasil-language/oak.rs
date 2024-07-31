//! `type` at the statement head is an alias only when a name follows it, and
//! import attribute clauses (`with { … }` / legacy `assert { … }`) stay inside
//! the import/export declaration.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{Statement, TypeScriptRoot},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

#[test]
fn assignment_to_variable_named_type_is_an_expression() {
    let root = build("type = strat.slice(6);");
    assert_eq!(root.statements.len(), 1);
    assert!(matches!(root.statements[0], Statement::ExpressionStatement(_)), "{:?}", root.statements);
}

#[test]
fn single_statement_if_body_can_assign_to_type() {
    let root = build("if (c) type = strat.slice(6);");
    let Some(Statement::IfStatement(if_stmt)) = root.statements.first() else {
        panic!("expected an if statement: {:?}", root.statements)
    };
    assert!(matches!(*if_stmt.consequent, Statement::ExpressionStatement(_)), "{:?}", if_stmt.consequent);
}

#[test]
fn type_alias_head_still_parses_as_alias() {
    let root = build("type Foo = string;");
    assert!(matches!(root.statements.first(), Some(Statement::TypeAlias(_))), "{:?}", root.statements);
}

#[test]
fn import_with_attributes_keeps_specifier() {
    let root = build("import { data } from './m.json' with { type: 'json' };");
    let Some(Statement::ImportDeclaration(import)) = root.statements.first() else {
        panic!("expected an import declaration: {:?}", root.statements)
    };
    assert_eq!(import.module_specifier.value, "./m.json");
    let attributes = import.attributes.as_deref().expect("attributes");
    assert!(attributes.starts_with("with"), "{attributes}");
    assert!(attributes.contains("json"), "{attributes}");
}

#[test]
fn bare_and_assert_imports_keep_attributes() {
    let root = build("import './m.json' assert { type: 'json' };");
    let Some(Statement::ImportDeclaration(import)) = root.statements.first() else {
        panic!("expected an import declaration: {:?}", root.statements)
    };
    assert_eq!(import.module_specifier.value, "./m.json");
    assert!(import.attributes.as_deref().is_some_and(|text| text.starts_with("assert")), "{:?}", import.attributes);

    let root = build("import def, { named } from './m.ts' with { type: 'module' };");
    let Some(Statement::ImportDeclaration(import)) = root.statements.first() else {
        panic!("expected an import declaration: {:?}", root.statements)
    };
    assert_eq!(import.module_specifier.value, "./m.ts");
    assert_eq!(import.specifiers.len(), 2);
    assert!(import.attributes.as_deref().is_some_and(|text| text.starts_with("with")), "{:?}", import.attributes);
}

#[test]
fn export_from_with_attributes_keeps_source() {
    let root = build("export { a } from './m.json' with { type: 'json' };");
    let Some(Statement::ExportDeclaration(export)) = root.statements.first() else {
        panic!("expected an export declaration: {:?}", root.statements)
    };
    assert_eq!(export.source.as_ref().expect("source").value, "./m.json");
    assert!(export.attributes.as_deref().is_some_and(|text| text.starts_with("with")), "{:?}", export.attributes);
}
