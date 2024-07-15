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

/// Returns the `OVER (...)` clause of a buildable window call.
fn window_of(expr: &Expression) -> &WindowSpec {
    match expr {
        Expression::FunctionCall { window: Some(window), .. } => window,
        other => panic!("Expected a window function call, got {other:?}"),
    }
}

#[test]
fn test_window_row_number_partitions_and_orders() {
    let (select, source) = select_and_source("SELECT ROW_NUMBER() OVER (PARTITION BY a ORDER BY b) FROM t;");
    let Expression::FunctionCall { name, args, distinct, window, .. } = first_expression(&select)
    else {
        panic!("Expected a function call, got {:?}", first_expression(&select));
    };
    assert_eq!(name.name.as_ref(), "ROW_NUMBER");
    assert!(args.is_empty(), "ROW_NUMBER() must stay argument-free: {args:?}");
    assert!(!distinct);
    let window = window.as_ref().unwrap_or_else(|| panic!("ROW_NUMBER() lost its window"));
    assert_eq!(window.partition_by.len(), 1);
    assert_eq!(window.partition_by[0].name.as_ref(), "a");
    assert_eq!(window.order_by.len(), 1);
    assert_eq!(window.order_by[0].column.name.as_ref(), "b");
    assert_eq!(window.order_by[0].direction, OrderDirection::Asc);
    assert_eq!(window.order_by[0].nulls, None);
    assert!(window.frame.is_none(), "no frame was written: {:?}", window.frame);
    assert_eq!(source, "SELECT ROW_NUMBER() OVER (PARTITION BY a ORDER BY b ASC)FROM t;");
}

#[test]
fn test_window_rank_orders_without_partition() {
    let (select, source) = select_and_source("SELECT RANK() OVER (ORDER BY a) FROM t;");
    let window = window_of(first_expression(&select));
    assert!(window.partition_by.is_empty(), "no `PARTITION BY` was written: {:?}", window.partition_by);
    assert_eq!(window.order_by.len(), 1);
    assert_eq!(window.order_by[0].column.name.as_ref(), "a");
    assert!(window.frame.is_none());
    assert_eq!(source, "SELECT RANK() OVER (ORDER BY a ASC)FROM t;");
}

#[test]
fn test_window_sum_rows_frame_between_unbounded_preceding_and_current_row() {
    let sql = "SELECT SUM(x) OVER (PARTITION BY a ORDER BY b ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) FROM t;";
    let (select, source) = select_and_source(sql);
    let window = window_of(first_expression(&select));
    assert_eq!(window.partition_by.len(), 1);
    assert_eq!(window.order_by.len(), 1);
    let frame = window.frame.as_ref().unwrap_or_else(|| panic!("SUM(x) lost its frame"));
    assert_eq!(frame.unit, WindowFrameUnit::Rows);
    assert_eq!(frame.start, WindowFrameBound::UnboundedPreceding);
    assert_eq!(frame.end, Some(WindowFrameBound::CurrentRow));
    assert_eq!(source, "SELECT SUM(x) OVER (PARTITION BY a ORDER BY b ASC ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW)FROM t;");
}

#[test]
fn test_window_frame_accepts_offsets_and_ranges() {
    let (select, _) = select_and_source("SELECT AVG(x) OVER (ORDER BY b ROWS 2 PRECEDING) FROM t;");
    let frame = window_of(first_expression(&select)).frame.clone().unwrap_or_else(|| panic!("AVG(x) lost its frame"));
    assert_eq!(frame.unit, WindowFrameUnit::Rows);
    assert_eq!(frame.start, WindowFrameBound::Preceding(2));
    assert_eq!(frame.end, None);

    let (select, _) = select_and_source("SELECT AVG(x) OVER (ORDER BY b RANGE BETWEEN 1 PRECEDING AND UNBOUNDED FOLLOWING) FROM t;");
    let frame = window_of(first_expression(&select)).frame.clone().unwrap_or_else(|| panic!("AVG(x) lost its frame"));
    assert_eq!(frame.unit, WindowFrameUnit::Range);
    assert_eq!(frame.start, WindowFrameBound::Preceding(1));
    assert_eq!(frame.end, Some(WindowFrameBound::UnboundedFollowing));
}

#[test]
fn test_plain_function_calls_keep_an_empty_window() {
    let (select, source) = select_and_source("SELECT COUNT(*) FROM t;");
    let Expression::FunctionCall { name, args, window, .. } = first_expression(&select)
    else {
        panic!("Expected a function call, got {:?}", first_expression(&select));
    };
    assert_eq!(name.name.as_ref(), "COUNT");
    assert!(matches!(args.as_slice(), [Expression::Wildcard { .. }]), "COUNT(*) must keep its wildcard: {args:?}");
    assert!(window.is_none(), "a plain call has no window: {window:?}");
    assert_eq!(source, "SELECT COUNT(*)FROM t;");

    let (select, _) = select_and_source("SELECT LOWER(name) FROM t;");
    assert!(window_of_none(first_expression(&select)), "LOWER(name) must stay a plain call");
}

/// Returns whether `expr` is a function call without a window.
fn window_of_none(expr: &Expression) -> bool {
    matches!(expr, Expression::FunctionCall { window: None, .. })
}

#[test]
fn test_window_rejects_empty_specification() {
    let (_, diagnostics) = build("SELECT ROW_NUMBER() OVER () FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("`OVER` requires")), "{diagnostics:?}");
}

#[test]
fn test_window_rejects_non_column_expressions() {
    let (_, diagnostics) = build("SELECT SUM(x) OVER (PARTITION BY a + 1) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("accepts only column references")), "{diagnostics:?}");

    let (_, diagnostics) = build("SELECT SUM(x) OVER (ORDER BY a * 2) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("accepts only column references")), "{diagnostics:?}");
}

#[test]
fn test_window_rejects_duplicate_partition_by() {
    let (_, diagnostics) = build("SELECT SUM(x) OVER (PARTITION BY a PARTITION BY b) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("Duplicate `PARTITION BY`")), "{diagnostics:?}");
}

#[test]
fn test_window_rejects_unsupported_frame_bounds() {
    let (_, diagnostics) = build("SELECT SUM(x) OVER (ORDER BY a ROWS BETWEEN UNBOUNDED FOLLOWING AND CURRENT ROW) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("cannot start a window frame")), "{diagnostics:?}");

    let (_, diagnostics) = build("SELECT SUM(x) OVER (ORDER BY a ROWS BETWEEN CURRENT ROW AND UNBOUNDED PRECEDING) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("cannot start after its end bound")), "{diagnostics:?}");

    let (_, diagnostics) = build("SELECT SUM(x) OVER (ORDER BY a ROWS 1 EXCLUDE TIES) FROM t;");
    assert!(diagnostics.iter().any(|message| message.contains("`EXCLUDE` is not supported")), "{diagnostics:?}");
}
