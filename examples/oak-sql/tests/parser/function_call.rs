use oak_core::{ParseSession, SourceText, builder::Builder, source::ToSource};
use oak_sql::{SqlBuilder, SqlLanguage, ast::*};

/// Builds `sql` and returns the root, if any, with the diagnostics it produced.
fn build(sql: &str) -> (Option<SqlRoot>, Vec<String>) {
    let language = SqlLanguage::default();
    let builder = SqlBuilder::new(&language);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new(sql), &[], &mut session);
    let diagnostics = result.diagnostics.iter().map(|error| error.to_string()).collect();
    (result.result.ok(), diagnostics)
}

/// Builds `sql`, asserts it is clean, and returns the select statement with the
/// source it round-trips to.
fn select_and_source(sql: &str) -> (SelectStatement, String) {
    let (root, diagnostics) = build(sql);
    let root = root.unwrap_or_else(|| panic!("{sql}: {diagnostics:?}"));
    assert!(diagnostics.is_empty(), "{sql}: {diagnostics:?}");
    let statement = root.statements.into_iter().next().unwrap_or_else(|| panic!("Missing statement for {sql}"));
    let SqlStatement::Select(select) = statement
    else {
        panic!("Expected a SELECT statement for {sql}, got {statement:?}");
    };
    let source = SqlRoot { statements: vec![SqlStatement::Select(select.clone())], span: root.span }.to_source_string();
    (select, source)
}

/// Returns the first select item expression.
fn first_expression(select: &SelectStatement) -> &Expression {
    match &select.items[0] {
        SelectItem::Expression { expr, .. } => expr,
        other => panic!("Expected an expression select item, got {other:?}"),
    }
}

/// Returns the call with the expected name, failing with its actual shape.
fn call<'a>(expr: &'a Expression, expected: &str) -> (&'a Identifier, &'a [Expression], Option<&'a WindowSpec>) {
    match expr {
        Expression::FunctionCall { name, args, window, .. } if name.name.as_ref() == expected => (name, args.as_slice(), window.as_ref()),
        other => panic!("Expected a `{expected}` call, got {other:?}"),
    }
}

#[test]
fn test_nested_call_argument_is_lowered_as_an_argument() {
    let (select, source) = select_and_source("SELECT SUM(LENGTH(email)) FROM users;");
    let (_, args, window) = call(first_expression(&select), "SUM");
    assert_eq!(args.len(), 1, "`SUM(LENGTH(email))` must keep its argument: {args:?}");
    let (_, inner_args, inner_window) = call(&args[0], "LENGTH");
    assert_eq!(inner_args.len(), 1, "`LENGTH(email)` must keep `email`: {inner_args:?}");
    let Expression::Identifier(email) = &inner_args[0]
    else {
        panic!("Expected `email` as an identifier argument, got {:?}", inner_args[0]);
    };
    assert_eq!(email.name.as_ref(), "email");
    assert!(inner_window.is_none(), "the nested call has no window: {inner_window:?}");
    assert!(window.is_none(), "`SUM` has no window: {window:?}");
    assert!(source.contains("SUM(LENGTH(email))"), "round-trip lost the nested call: {source}");
}

#[test]
fn test_nested_call_without_arguments() {
    let (select, source) = select_and_source("SELECT CHAR_LENGTH(NOW()) FROM users;");
    let (_, args, _) = call(first_expression(&select), "CHAR_LENGTH");
    assert_eq!(args.len(), 1, "`CHAR_LENGTH(NOW())` must keep its argument: {args:?}");
    let (_, inner_args, _) = call(&args[0], "NOW");
    assert!(inner_args.is_empty(), "`NOW()` takes no arguments: {inner_args:?}");
    assert!(source.contains("CHAR_LENGTH(NOW())"), "round-trip lost the nested call: {source}");
}

#[test]
fn test_multiple_nested_call_arguments_keep_their_order() {
    let (select, source) = select_and_source("SELECT COALESCE(LOWER(name), UPPER(name)) FROM users;");
    let (_, args, _) = call(first_expression(&select), "COALESCE");
    assert_eq!(args.len(), 2, "`COALESCE` must keep both arguments: {args:?}");
    for (arg, expected) in args.iter().zip(["LOWER", "UPPER"]) {
        let (_, inner_args, _) = call(arg, expected);
        assert_eq!(inner_args.len(), 1, "`{expected}(name)` must keep `name`: {inner_args:?}");
    }
    assert!(source.contains("COALESCE(LOWER(name),UPPER(name))"), "round-trip lost the argument order: {source}");
}

#[test]
fn test_plain_call_shapes_do_not_regress() {
    let (select, source) = select_and_source("SELECT COUNT(*) FROM t;");
    let (_, args, window) = call(first_expression(&select), "COUNT");
    assert!(matches!(args, [Expression::Wildcard { .. }]), "`COUNT(*)` must keep its wildcard: {args:?}");
    assert!(window.is_none(), "`COUNT(*)` has no window: {window:?}");
    assert!(source.contains("COUNT(*)"), "round-trip lost the wildcard: {source}");

    let (select, source) = select_and_source("SELECT NOW() FROM t;");
    let (_, args, _) = call(first_expression(&select), "NOW");
    assert!(args.is_empty(), "`NOW()` takes no arguments: {args:?}");
    assert!(source.contains("NOW()"), "round-trip lost the call: {source}");

    let (select, _) = select_and_source("SELECT ROUND(x, 2) FROM t;");
    let (_, args, window) = call(first_expression(&select), "ROUND");
    assert_eq!(args.len(), 2, "`ROUND(x, 2)` must keep both arguments: {args:?}");
    assert!(matches!(&args[0], Expression::Identifier(identifier) if identifier.name.as_ref() == "x"), "the first argument stays an identifier: {:?}", args[0]);
    assert!(matches!(&args[1], Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "2"), "the second argument stays a number literal: {:?}", args[1]);
    assert!(window.is_none(), "`ROUND(x, 2)` has no window: {window:?}");
}

#[test]
fn test_window_call_shape_does_not_regress() {
    let (select, source) = select_and_source("SELECT ROW_NUMBER() OVER (ORDER BY id) FROM t;");
    let (_, args, window) = call(first_expression(&select), "ROW_NUMBER");
    assert!(args.is_empty(), "`ROW_NUMBER()` takes no arguments: {args:?}");
    let window = window.unwrap_or_else(|| panic!("`ROW_NUMBER()` lost its window"));
    assert!(window.partition_by.is_empty());
    assert_eq!(window.order_by.len(), 1);
    assert_eq!(window.order_by[0].column.name.as_ref(), "id");
    assert!(source.contains("ROW_NUMBER() OVER (ORDER BY id ASC)"), "round-trip lost the window: {source}");
}

#[test]
fn test_window_still_rejects_nested_calls_in_its_clauses() {
    let (_, diagnostics) = build("SELECT SUM(x) OVER (ORDER BY LOWER(a)) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("accepts only column references")), "{diagnostics:?}");

    let (_, diagnostics) = build("SELECT SUM(x) OVER (PARTITION BY UPPER(a)) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("accepts only column references")), "{diagnostics:?}");
}
