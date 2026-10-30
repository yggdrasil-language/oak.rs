//! Generator declarations keep the `*` inside the declaration node, so
//! `function*` and `async function*` no longer report an expected `LeftParen`
//! at the name.

use oak_core::{Builder, SourceText, parser::ParseSession};
use oak_typescript::{
    TypeScriptBuilder, TypeScriptLanguage,
    ast::{FunctionDeclaration, Statement, TypeScriptRoot},
};

fn build(source: &str) -> TypeScriptRoot {
    let lang = TypeScriptLanguage::standard();
    let mut cache = ParseSession::<TypeScriptLanguage>::new(32);
    let result = TypeScriptBuilder::new(&lang).build(&SourceText::new(source), &[], &mut cache);
    assert!(result.diagnostics.is_empty(), "{source}: {:#?}", result.diagnostics);
    result.result.unwrap_or_else(|e| panic!("{source}: build failed: {e:?}"))
}

fn function_decl(root: &TypeScriptRoot) -> &FunctionDeclaration {
    let Statement::FunctionDeclaration(decl) = &root.statements[0]
    else {
        panic!("expected function declaration: {:?}", root.statements)
    };
    decl
}

#[test]
fn generator_declaration_parses_yield_body() {
    let source = "function* g(){yield 1}";
    let root = build(source);
    let decl = function_decl(&root);

    assert_eq!(decl.name, "g");
    assert!(!decl.body.is_empty(), "generator body must keep its statements");
    assert_eq!(&source[decl.span.start..decl.span.end], source);
}

#[test]
fn async_generator_declaration_parses_yield_star() {
    let source = "async function* g(){yield* x}";
    let root = build(source);
    let decl = function_decl(&root);

    assert_eq!(decl.name, "g");
    // `yield*` expressions are not modeled as `YieldExpression` yet, so only the
    // declaration head is pinned here.
    assert_eq!(&source[decl.span.start..decl.span.end], source);
}

#[test]
fn generator_function_expression_parses() {
    let source = "const f = function* (){yield 1};";
    let root = build(source);

    let Statement::VariableDeclaration(decl) = &root.statements[0]
    else {
        panic!("expected variable declaration: {:?}", root.statements)
    };
    assert_eq!(decl.name, "f");
    assert_eq!(root.statements.len(), 1);
}
