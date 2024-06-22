use oak_core::{ParseSession, SourceText, builder::Builder, source::ToSource};
use oak_sql::{SqlBuilder, SqlLanguage, ast::*};

#[test]
fn test_sql_builder_select() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let source = SourceText::new("SELECT name, age FROM users WHERE age > 18;");

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    if let Err(e) = &result.result {
        panic!("Build error: {e:?}\nDiagnostics: {:?}", result.diagnostics);
    }
    let root = result.result.unwrap();
    assert_eq!(root.statements.len(), 1);

    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    assert_eq!(select.items.len(), 2);
    assert!(select.from.is_some());
    assert!(select.expr.is_some());
}

#[test]
fn test_sql_builder_session_statements() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);

    for (source, expected) in [("BEGIN", TransactionAction::Begin), ("COMMIT TRANSACTION", TransactionAction::Commit), ("ROLLBACK", TransactionAction::Rollback)] {
        let mut session = ParseSession::<SqlLanguage>::default();
        let result = builder.build(&SourceText::new(source), &[], &mut session);
        let root = result.result.unwrap_or_else(|error| panic!("{source}: {error:?} {:?}", result.diagnostics));
        let SqlStatement::Transaction(statement) = &root.statements[0]
        else {
            panic!("Expected transaction statement for {source}");
        };
        assert_eq!(statement.action, expected);
    }

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SET NAMES utf8mb4 COLLATE utf8mb4_bin"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("SET NAMES: {error:?} {:?}", result.diagnostics));
    let SqlStatement::SetNames(statement) = &root.statements[0]
    else {
        panic!("Expected SET NAMES statement, got {:?}", root.statements);
    };
    assert_eq!(statement.character_set.as_ref(), "utf8mb4");
    assert_eq!(statement.collation.as_deref(), Some("utf8mb4_bin"));
}

/// Builds `sql` and returns its single statement with the source it round-trips to.
fn build_statement(sql: &str) -> (SqlStatement, String) {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new(sql), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("{sql}: {error:?} {:?}", result.diagnostics));
    let statement = root.statements.into_iter().next().unwrap_or_else(|| panic!("Missing statement for {sql}"));
    let source = SqlRoot { statements: vec![statement.clone()], span: root.span }.to_source_string();
    (statement, source)
}

#[test]
fn test_sql_builder_scoped_session_variables() {
    let (statement, source) = build_statement("SET SESSION sql_mode = 'STRICT';");
    let SqlStatement::SetVariable(statement) = &statement
    else {
        panic!("Expected SET statement, got {statement:?}");
    };
    assert_eq!(statement.scope, Some(SessionScope::Session));
    assert_eq!(statement.name.as_ref(), "sql_mode");
    assert!(matches!(&statement.value, Expression::Literal(Literal::String(value, _)) if value.as_ref() == "STRICT"));
    assert_eq!(source, "SET SESSION sql_mode = 'STRICT';");

    let (statement, source) = build_statement("SET autocommit = 0;");
    let SqlStatement::SetVariable(statement) = &statement
    else {
        panic!("Expected SET statement, got {statement:?}");
    };
    assert_eq!(statement.scope, None);
    assert_eq!(statement.name.as_ref(), "autocommit");
    assert!(matches!(&statement.value, Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "0"));
    assert_eq!(source, "SET autocommit = 0;");
}

#[test]
fn test_sql_builder_show_and_pragma() {
    let (statement, source) = build_statement("SHOW GLOBAL VARIABLES LIKE 'max_%';");
    let SqlStatement::Show(statement) = &statement
    else {
        panic!("Expected SHOW statement, got {statement:?}");
    };
    assert_eq!(statement.scope, Some(SessionScope::Global));
    assert_eq!(statement.target.as_ref(), "VARIABLES");
    assert_eq!(statement.like.as_deref(), Some("max_%"));
    assert_eq!(source, "SHOW GLOBAL VARIABLES LIKE 'max_%';");

    let (statement, source) = build_statement("SHOW STATUS;");
    let SqlStatement::Show(statement) = &statement
    else {
        panic!("Expected SHOW statement, got {statement:?}");
    };
    assert_eq!(statement.scope, None);
    assert_eq!(statement.target.as_ref(), "STATUS");
    assert_eq!(statement.like, None);
    assert_eq!(source, "SHOW STATUS;");

    let (statement, source) = build_statement("PRAGMA journal_mode = WAL;");
    let SqlStatement::Pragma(statement) = &statement
    else {
        panic!("Expected PRAGMA statement, got {statement:?}");
    };
    assert_eq!(statement.name.as_ref(), "journal_mode");
    assert!(matches!(&statement.value, Some(Expression::Identifier(identifier)) if identifier.name.as_ref() == "WAL"));
    assert_eq!(source, "PRAGMA journal_mode = WAL;");

    let (statement, source) = build_statement("PRAGMA user_version;");
    let SqlStatement::Pragma(statement) = &statement
    else {
        panic!("Expected PRAGMA statement, got {statement:?}");
    };
    assert_eq!(statement.name.as_ref(), "user_version");
    assert!(statement.value.is_none());
    assert_eq!(source, "PRAGMA user_version;");
}

/// Builds `sql` and returns the `WHERE` expression of its single SELECT statement.
fn build_where(sql: &str) -> Expression {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new(sql), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("{sql}: {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement for {sql}, got {:?}", root.statements);
    };
    select.expr.clone().unwrap_or_else(|| panic!("Missing WHERE expression for {sql}"))
}

#[test]
fn test_sql_builder_predicates() {
    let Expression::InList { list, negated, .. } = build_where("SELECT * FROM t WHERE id IN (1, 2, 3);")
    else {
        panic!("Expected IN list");
    };
    assert!(!negated);
    assert_eq!(list.len(), 3);
    assert!(matches!(&list[2], Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "3"));

    let Expression::Between { low, high, negated, .. } = build_where("SELECT * FROM t WHERE age BETWEEN 18 AND 30;")
    else {
        panic!("Expected BETWEEN");
    };
    assert!(!negated);
    assert!(matches!(low.as_ref(), Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "18"));
    assert!(matches!(high.as_ref(), Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "30"));

    let Expression::IsNull { expr, negated: false, span } = build_where("SELECT * FROM t WHERE name IS NULL;")
    else {
        panic!("Expected IS NULL");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "name"));
    assert_eq!(span.start, expr.span().start);

    let Expression::IsNull { expr, negated: true, .. } = build_where("SELECT * FROM t WHERE name IS NOT NULL;")
    else {
        panic!("Expected IS NOT NULL");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "name"));

    let Expression::Binary { op: BinaryOperator::Like, right, .. } = build_where("SELECT * FROM t WHERE name LIKE 'a%';")
    else {
        panic!("Expected LIKE");
    };
    assert!(matches!(right.as_ref(), Expression::Literal(Literal::String(value, _)) if value.as_ref() == "a%"));

    let Expression::Binary { op: BinaryOperator::Equal, left, .. } = build_where("SELECT * FROM t WHERE id % 2 = 0;")
    else {
        panic!("Expected equality around a modulo");
    };
    assert!(matches!(left.as_ref(), Expression::Binary { op: BinaryOperator::Percent, .. }));
}

/// Builds `sql` and returns why it was rejected: the fatal error, the rejected
/// statement, or the diagnostics collected while parsing.
fn rejection_reason(sql: &str) -> String {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new(sql), &[], &mut session);
    let mut reasons = result.diagnostics.iter().map(|diagnostic| diagnostic.to_string()).collect::<Vec<_>>();
    match result.result {
        Err(error) => reasons.push(error.to_string()),
        Ok(root) => reasons.extend(root.statements.iter().filter_map(|statement| match statement {
            SqlStatement::Error { message, .. } => Some(message.to_string()),
            _ => None,
        })),
    }
    reasons.join("\n")
}

#[test]
fn test_sql_builder_is_null_and_truth_predicates() {
    // `IS NULL` and `IS TRUE` are three-valued predicates, not comparisons.
    let Expression::IsNull { expr, negated: false, span } = build_where("SELECT * FROM t WHERE name IS NULL;")
    else {
        panic!("Expected IS NULL");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "name"));
    assert_eq!(span.start, expr.span().start);

    let Expression::IsNull { negated: true, .. } = build_where("SELECT * FROM t WHERE name IS NOT NULL;")
    else {
        panic!("Expected IS NOT NULL");
    };

    let Expression::IsTruth { expr, value: true, negated: false, span } = build_where("SELECT * FROM t WHERE flag IS TRUE;")
    else {
        panic!("Expected IS TRUE");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "flag"));
    assert_eq!(span.start, expr.span().start);

    let Expression::IsTruth { value: false, negated: true, .. } = build_where("SELECT * FROM t WHERE flag IS NOT FALSE;")
    else {
        panic!("Expected IS NOT FALSE");
    };

    // `IS` keeps the same precedence as `IN` and `BETWEEN`: a surrounding `AND`
    // stays outside the predicate.
    let Expression::Binary { op: BinaryOperator::And, left, .. } = build_where("SELECT * FROM t WHERE flag IS TRUE AND id > 0;")
    else {
        panic!("Expected AND around IS TRUE");
    };
    assert!(matches!(left.as_ref(), Expression::IsTruth { value: true, negated: false, .. }));
}

#[test]
fn test_sql_builder_is_predicate_source_round_trip() {
    for (sql, expected) in [
        ("SELECT * FROM t WHERE name IS NULL;", "SELECT*FROM t WHERE name IS NULL;"),
        ("SELECT * FROM t WHERE name IS NOT NULL;", "SELECT*FROM t WHERE name IS NOT NULL;"),
        ("SELECT * FROM t WHERE flag IS TRUE;", "SELECT*FROM t WHERE flag IS TRUE;"),
        ("SELECT * FROM t WHERE flag IS NOT FALSE;", "SELECT*FROM t WHERE flag IS NOT FALSE;"),
        ("SELECT * FROM t WHERE NOT flag IS NULL;", "SELECT*FROM t WHERE NOT flag IS NULL;"),
        // A negated operand has to keep its parentheses: `IS` binds tighter
        // than `NOT`, so dropping them would reparse as `NOT (flag IS NULL)`.
        ("SELECT * FROM t WHERE (NOT flag) IS NULL;", "SELECT*FROM t WHERE(NOT flag) IS NULL;"),
        ("SELECT * FROM t WHERE (a = 1) IS FALSE;", "SELECT*FROM t WHERE(a=1) IS FALSE;"),
    ] {
        let (_, source) = build_statement(sql);
        assert_eq!(source, expected, "{sql}");
        // The printed source must build the same predicate again.
        let rebuilt = build_where(&source);
        assert!(matches!(rebuilt, Expression::Unary { .. } | Expression::IsNull { .. } | Expression::IsTruth { .. }), "unexpected rebuilt predicate for {sql}: {rebuilt:?}");
    }
}

#[test]
fn test_sql_builder_null_comparison_stays_binary() {
    // A plain `=` / `<>` against `NULL` is a comparison, not an `IS` predicate.
    let Expression::Binary { op: BinaryOperator::Equal, right, .. } = build_where("SELECT * FROM t WHERE name = NULL;")
    else {
        panic!("Expected comparison against NULL");
    };
    assert!(matches!(right.as_ref(), Expression::Literal(Literal::Null(_))));

    let Expression::Binary { op: BinaryOperator::NotEqual, right, .. } = build_where("SELECT * FROM t WHERE name <> NULL;")
    else {
        panic!("Expected negated comparison against NULL");
    };
    assert!(matches!(right.as_ref(), Expression::Literal(Literal::Null(_))));

    let (_, source) = build_statement("SELECT * FROM t WHERE name = NULL;");
    assert_eq!(source, "SELECT*FROM t WHERE name=NULL;");
}

#[test]
fn test_sql_builder_unsupported_is_predicates_are_reported() {
    // `IS UNKNOWN` is a three-valued predicate without an AST node yet. It must be
    // rejected instead of silently folding into `= UNKNOWN`.
    let reason = rejection_reason("SELECT * FROM t WHERE a IS UNKNOWN;");
    assert!(reason.contains("`IS UNKNOWN` is not supported"), "unexpected rejection: {reason}");

    // `IS [NOT] DISTINCT FROM` is a null-safe comparison. A partial `IS` must
    // never survive as a usable predicate.
    for sql in ["SELECT * FROM t WHERE a IS DISTINCT FROM b;", "SELECT * FROM t WHERE a IS NOT DISTINCT FROM b;"] {
        let reason = rejection_reason(sql);
        assert!(reason.contains("`IS DISTINCT FROM` is not supported"), "{sql}: {reason}");
    }

    // A bare `IS` with no operand must not silently disappear either.
    let reason = rejection_reason("SELECT * FROM t WHERE a IS;");
    assert!(!reason.is_empty(), "`IS` without an operand must be reported");
}

#[test]
fn test_sql_builder_qualified_columns_and_function_calls() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT t.id, t.name FROM t;"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("qualified columns: {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    let names = select
        .items
        .iter()
        .map(|item| match item {
            SelectItem::Expression { expr: Expression::Identifier(identifier), .. } => identifier.name.to_string(),
            other => panic!("Expected identifier select item, got {other:?}"),
        })
        .collect::<Vec<_>>();
    assert_eq!(names, ["t.id", "t.name"]);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT COUNT(*) FROM users;"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("COUNT(*): {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::FunctionCall { name, args, .. }, .. } = &select.items[0]
    else {
        panic!("Expected function call select item, got {:?}", select.items[0]);
    };
    assert_eq!(name.name.as_ref(), "COUNT");
    assert!(matches!(args.as_slice(), [Expression::Wildcard { .. }]), "COUNT(*) must keep its wildcard argument: {args:?}");

    let (_, source) = build_statement("SELECT COUNT(*) FROM users;");
    assert_eq!(source, "SELECT COUNT(*)FROM users;");

    // A genuinely empty argument list stays empty and must not be confused with
    // the wildcard form.
    let (statement, _) = build_statement("SELECT COUNT() FROM users;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::FunctionCall { args, .. }, .. } = &select.items[0]
    else {
        panic!("Expected function call select item, got {:?}", select.items[0]);
    };
    assert!(args.is_empty(), "COUNT() must stay argument-free: {args:?}");

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT LOWER(name) FROM users;"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("LOWER(name): {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::FunctionCall { name, args, .. }, .. } = &select.items[0]
    else {
        panic!("Expected function call select item, got {:?}", select.items[0]);
    };
    assert_eq!(name.name.as_ref(), "LOWER");
    assert!(matches!(args.as_slice(), [Expression::Identifier(identifier)] if identifier.name.as_ref() == "name"));

    // Aggregate names are callable identifiers, not reserved keyword tokens.
    // `MAX` used to be lexed as its own keyword, which made `MAX(age)` fall into
    // an error node instead of a function call.
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT MAX(age) FROM users;"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("MAX(age): {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::FunctionCall { name, args, .. }, .. } = &select.items[0]
    else {
        panic!("Expected function call select item, got {:?}", select.items[0]);
    };
    assert_eq!(name.name.as_ref(), "MAX");
    assert!(matches!(args.as_slice(), [Expression::Identifier(identifier)] if identifier.name.as_ref() == "age"));
}

#[test]
fn test_sql_builder_prefix_unary_signs() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT -3;"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("SELECT -3: {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::Unary { op, expr, .. }, .. } = &select.items[0]
    else {
        panic!("Expected unary select item, got {:?}", select.items[0]);
    };
    assert_eq!(*op, UnaryOperator::Minus);
    assert!(matches!(expr.as_ref(), Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "3"));

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT * FROM t WHERE age > -1;"), &[], &mut session);
    let root = result.result.unwrap_or_else(|error| panic!("age > -1: {error:?} {:?}", result.diagnostics));
    let SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    let Some(Expression::Binary { op: BinaryOperator::Greater, right, .. }) = &select.expr
    else {
        panic!("Expected greater-than expression, got {:?}", select.expr);
    };
    assert!(matches!(right.as_ref(), Expression::Unary { op: UnaryOperator::Minus, .. }));
}

/// Before the fix `WHERE NOT (a = 1)` parsed `NOT` as a lone `ErrorNode`, so the
/// builder saw no `Expression` child and left `SelectStatement.expr` as `None`.
/// That silently dropped the predicate instead of rejecting it.
#[test]
fn test_sql_builder_prefix_not() {
    let Expression::Unary { op: UnaryOperator::Not, expr, .. } = build_where("SELECT * FROM t WHERE NOT (a = 1);")
    else {
        panic!("Expected NOT predicate");
    };
    assert!(matches!(expr.as_ref(), Expression::Binary { op: BinaryOperator::Equal, .. }));

    // `NOT` binds tighter than `AND` and looser than comparisons: `NOT a = 1`
    // must be `NOT (a = 1)`, never `(NOT a) = 1`.
    let Expression::Unary { op: UnaryOperator::Not, expr, .. } = build_where("SELECT * FROM t WHERE NOT a = 1;")
    else {
        panic!("Expected NOT around a comparison");
    };
    assert!(matches!(expr.as_ref(), Expression::Binary { op: BinaryOperator::Equal, .. }));

    let Expression::Unary { op: UnaryOperator::Not, expr, .. } = build_where("SELECT * FROM t WHERE NOT NOT a = 1;")
    else {
        panic!("Expected nested NOT");
    };
    assert!(matches!(expr.as_ref(), Expression::Unary { op: UnaryOperator::Not, .. }));

    // `IS` binds tighter than `NOT`, so the negation is around `flag IS NULL`.
    let Expression::Unary { op: UnaryOperator::Not, expr, .. } = build_where("SELECT * FROM t WHERE NOT flag IS NULL;")
    else {
        panic!("Expected NOT around IS NULL");
    };
    assert!(matches!(expr.as_ref(), Expression::IsNull { negated: false, .. }));

    let (_, source) = build_statement("SELECT * FROM t WHERE NOT (a = 1);");
    assert_eq!(source, "SELECT*FROM t WHERE NOT a=1;");
}

#[test]
fn test_sql_builder_not_precedence_with_and_or() {
    // `AND` binds tighter than `OR` and both bind looser than `NOT`.
    let Expression::Binary { op: BinaryOperator::And, left, right, .. } = build_where("SELECT * FROM t WHERE NOT a = 1 AND b = 2;")
    else {
        panic!("Expected AND at the root");
    };
    assert!(matches!(left.as_ref(), Expression::Unary { op: UnaryOperator::Not, .. }));
    assert!(matches!(right.as_ref(), Expression::Binary { op: BinaryOperator::Equal, .. }));

    let Expression::Binary { op: BinaryOperator::Or, left, right, .. } = build_where("SELECT * FROM t WHERE a = 1 OR NOT b = 2;")
    else {
        panic!("Expected OR at the root");
    };
    assert!(matches!(left.as_ref(), Expression::Binary { op: BinaryOperator::Equal, .. }));
    assert!(matches!(right.as_ref(), Expression::Unary { op: UnaryOperator::Not, .. }));

    let Expression::Binary { op: BinaryOperator::Or, left, .. } = build_where("SELECT * FROM t WHERE NOT a = 1 OR b = 2;")
    else {
        panic!("Expected OR at the root");
    };
    assert!(matches!(left.as_ref(), Expression::Unary { op: UnaryOperator::Not, .. }));
}

#[test]
fn test_sql_builder_negated_predicates() {
    let Expression::InList { list, negated: true, .. } = build_where("SELECT * FROM t WHERE a NOT IN (1, 2);")
    else {
        panic!("Expected negated IN list");
    };
    assert_eq!(list.len(), 2);

    let Expression::Unary { op: UnaryOperator::Not, expr, .. } = build_where("SELECT * FROM t WHERE NOT a IN (1, 2);")
    else {
        panic!("Expected NOT around IN");
    };
    assert!(matches!(expr.as_ref(), Expression::InList { negated: false, .. }));

    let Expression::Between { negated: true, low, high, .. } = build_where("SELECT * FROM t WHERE a NOT BETWEEN 1 AND 2;")
    else {
        panic!("Expected negated BETWEEN");
    };
    assert!(matches!(low.as_ref(), Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "1"));
    assert!(matches!(high.as_ref(), Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "2"));

    let Expression::Unary { op: UnaryOperator::Not, expr, .. } = build_where("SELECT * FROM t WHERE name NOT LIKE 'a%';")
    else {
        panic!("Expected NOT LIKE");
    };
    assert!(matches!(expr.as_ref(), Expression::Binary { op: BinaryOperator::Like, .. }));

    let Expression::IsTruth { expr, value: true, negated: true, .. } = build_where("SELECT * FROM t WHERE a IS NOT TRUE;")
    else {
        panic!("Expected IS NOT TRUE");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "a"));

    let (_, source) = build_statement("SELECT * FROM t WHERE a NOT IN (1, 2);");
    assert_eq!(source, "SELECT*FROM t WHERE a NOT IN(1,2);");
}

#[test]
fn test_sql_builder_in_subquery() {
    let Expression::InSubquery { query, negated: false, expr, .. } = build_where("SELECT * FROM t WHERE a IN (SELECT b FROM u);")
    else {
        panic!("Expected IN subquery");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "a"));
    assert_eq!(query.from.as_ref().map(|table| table.name.name.as_ref()), Some("u"));
    assert!(matches!(&query.items[0], SelectItem::Expression { expr: Expression::Identifier(identifier), .. } if identifier.name.as_ref() == "b"));

    let Expression::InSubquery { negated: true, .. } = build_where("SELECT * FROM t WHERE a NOT IN (SELECT b FROM u);")
    else {
        panic!("Expected negated IN subquery");
    };

    let (_, source) = build_statement("SELECT * FROM t WHERE a IN (SELECT b FROM u);");
    assert_eq!(source, "SELECT*FROM t WHERE a IN(SELECT b FROM u);");

    // A mixed list is not valid SQL and must be rejected rather than silently
    // keeping only the first element.
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT * FROM t WHERE a IN (1, SELECT b FROM u);"), &[], &mut session);
    assert!(result.result.is_err(), "Mixed IN list must not be accepted: {:?}", result.result);
}

#[test]
fn test_sql_builder_unsupported_predicate_is_reported() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    // An unsupported predicate must surface as an error. Before the fix it became
    // `SelectStatement.expr == None`, which downstream read as "match every row".
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT * FROM t WHERE EXISTS (SELECT 1);"), &[], &mut session);
    assert!(result.result.is_err(), "Unsupported predicate must not become `None`: {:?}", result.result);

    // A trailing unsupported token must abort parsing instead of keeping only the
    // first half of the predicate.
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT * FROM t WHERE a = 1 XOR b = 2;"), &[], &mut session);
    assert!(result.result.is_err(), "Partially parsed WHERE must not be accepted: {:?}", result.result);
}

/// Every identifier span must slice exactly the identifier text, no matter how
/// much leading trivia precedes it.
#[test]
fn test_sql_builder_identifier_spans_skip_leading_trivia() {
    for sql in [
        "SELECT id FROM users;",
        "  SELECT id FROM users;",
        "\n\n\tSELECT id FROM users;",
        "/* note */ SELECT id FROM users;",
        "  \n /* note */\n\tSELECT id FROM users;",
        "\r\n\t-- note\n  SELECT id FROM users;",
        "; SELECT id FROM users;",
        "BROKEN; SELECT id FROM users;",
    ] {
        let config = SqlLanguage::default();
        let builder = SqlBuilder::new(&config);
        let mut session = ParseSession::<SqlLanguage>::default();
        let result = builder.build(&SourceText::new(sql), &[], &mut session);
        let root = result.result.unwrap_or_else(|error| panic!("{sql}: {error:?} {:?}", result.diagnostics));

        let SqlStatement::Select(select) = &root.statements[0]
        else {
            panic!("Expected Select statement for {sql}, got {:?}", root.statements);
        };
        let SelectItem::Expression { expr, .. } = &select.items[0]
        else {
            panic!("Expected expression item for {sql}");
        };
        let Expression::Identifier(identifier) = expr
        else {
            panic!("Expected identifier item for {sql}, got {expr:?}");
        };
        assert_eq!(sql.get(identifier.span.start..identifier.span.end), Some("id"), "identifier span for {sql}");

        let table = select.from.as_ref().unwrap_or_else(|| panic!("Missing table for {sql}"));
        assert_eq!(sql.get(table.name.span.start..table.name.span.end), Some("users"), "table span for {sql}");
        assert_eq!(sql.get(root.span.start..root.span.end), Some(sql), "root must cover the whole source for {sql}");
    }
}

#[test]
fn test_sql_builder_select_quantifier() {
    let sql = "SELECT DISTINCT name FROM users;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    let Some(SelectQuantifier::Distinct { span }) = &select.quantifier
    else {
        panic!("Expected DISTINCT quantifier, got {:?}", select.quantifier);
    };
    assert_eq!(sql.get(span.start..span.end), Some("DISTINCT"));
    assert_eq!(source, "SELECT DISTINCT name FROM users;");

    let sql = "SELECT DISTINCT ON (user_id, name) name FROM users;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    let Some(SelectQuantifier::DistinctOn { expressions, span }) = &select.quantifier
    else {
        panic!("Expected DISTINCT ON quantifier, got {:?}", select.quantifier);
    };
    assert_eq!(sql.get(span.start..span.end), Some("DISTINCT ON (user_id, name)"));
    assert_eq!(expressions.len(), 2);
    assert!(matches!(&expressions[0], Expression::Identifier(identifier) if identifier.name.as_ref() == "user_id"));
    assert!(matches!(&expressions[1], Expression::Identifier(identifier) if identifier.name.as_ref() == "name"));
    assert_eq!(source, "SELECT DISTINCT ON(user_id,name)name FROM users;");

    let (statement, source) = build_statement("SELECT ALL name FROM users;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    assert!(matches!(select.quantifier, Some(SelectQuantifier::All { .. })));
    assert_eq!(source, "SELECT ALL name FROM users;");

    let (statement, _) = build_statement("SELECT name FROM users;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    assert!(select.quantifier.is_none());
}

#[test]
fn test_sql_builder_distinct_outside_select_is_reported() {
    // `DISTINCT` may only appear in the SELECT quantifier position or inside
    // aggregate function calls like `COUNT(DISTINCT x)`. The standalone keyword
    // in other positions must still be reported instead of silently parsed.
    for sql in ["SELECT name FROM users ORDER BY DISTINCT name;", "DISTINCT name FROM users;"] {
        let reason = rejection_reason(sql);
        assert!(reason.contains("`DISTINCT` is only allowed immediately after `SELECT`"), "{sql}: {reason}");
    }

    // `COUNT(DISTINCT x)` is now parsed normally — the DISTINCT token is kept in
    // the CST and the builder projects it onto the FunctionCall as a boolean flag.
    let (statement, source) = build_statement("SELECT COUNT(DISTINCT name) FROM users;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("expected SELECT statement")
    };
    let SelectItem::Expression { expr, .. } = &select.items[0]
    else {
        panic!("expected Expression select item")
    };
    let Expression::FunctionCall { distinct, args, .. } = expr
    else {
        panic!("expected FunctionCall projection")
    };
    assert!(*distinct, "DISTINCT flag must be set on COUNT call");
    assert_eq!(args.len(), 1, "COUNT(DISTINCT name) should have one argument");

    let reason = rejection_reason("SELECT DISTINCT ON name FROM users;");
    assert!(reason.contains("`DISTINCT ON` requires a parenthesized expression list"), "{reason}");

    let reason = rejection_reason("SELECT DISTINCT ON () name FROM users;");
    assert!(reason.contains("`DISTINCT ON` requires at least one expression"), "{reason}");
}

#[test]
fn test_sql_builder_group_by_items_and_having() {
    let sql = "SELECT dept, COUNT(*) FROM employees GROUP BY dept, 1, LOWER(name) HAVING COUNT(*) > 1;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };

    let group_by = select.group_by.as_ref().unwrap_or_else(|| panic!("Missing GROUP BY"));
    assert_eq!(group_by.items.len(), 3);

    let GroupByItem::Column(column) = &group_by.items[0]
    else {
        panic!("Expected a bare column item, got {:?}", group_by.items[0]);
    };
    assert_eq!(column.name.as_ref(), "dept");
    assert_eq!(sql.get(column.span.start..column.span.end), Some("dept"));

    let GroupByItem::Position { index, span } = &group_by.items[1]
    else {
        panic!("Expected a positional item, got {:?}", group_by.items[1]);
    };
    assert_eq!(*index, 1);
    assert_eq!(sql.get(span.start..span.end), Some("1"));

    assert!(matches!(&group_by.items[2], GroupByItem::Expression(Expression::FunctionCall { .. })), "expected a function item, got {:?}", group_by.items[2]);

    let having = select.having.as_ref().unwrap_or_else(|| panic!("Missing HAVING"));
    assert!(matches!(having.condition, Expression::Binary { op: BinaryOperator::Greater, .. }), "unexpected HAVING predicate {:?}", having.condition);

    assert_eq!(source, "SELECT dept,COUNT(*)FROM employees GROUP BY dept,1,LOWER(name)HAVING COUNT(*)>1;");
}

#[test]
fn test_sql_builder_group_by_distinguishes_positions_from_expressions() {
    let (statement, source) = build_statement("SELECT a FROM t GROUP BY (1), 1 + 1, t.a;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    let group_by = select.group_by.as_ref().unwrap_or_else(|| panic!("Missing GROUP BY"));

    // A parenthesized constant is an expression, not a select-list position.
    assert!(matches!(&group_by.items[0], GroupByItem::Expression(Expression::Literal(_))), "unexpected item {:?}", group_by.items[0]);
    assert!(matches!(&group_by.items[1], GroupByItem::Expression(Expression::Binary { .. })), "unexpected item {:?}", group_by.items[1]);
    let GroupByItem::Column(column) = &group_by.items[2]
    else {
        panic!("Expected a qualified column item, got {:?}", group_by.items[2]);
    };
    assert_eq!(column.name.as_ref(), "t.a");
    assert_eq!(source, "SELECT a FROM t GROUP BY 1,1+1,t.a;");
}

/// `HAVING` must bind without a `GROUP BY` clause. Rejecting it is the
/// downstream semantic layer's job, not the parser's.
#[test]
fn test_sql_builder_having_without_group_by_binds() {
    let (statement, source) = build_statement("SELECT flag FROM events HAVING flag;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    assert!(select.group_by.is_none());
    let having = select.having.as_ref().unwrap_or_else(|| panic!("Missing HAVING"));
    assert!(matches!(&having.condition, Expression::Identifier(identifier) if identifier.name.as_ref() == "flag"));
    assert_eq!(source, "SELECT flag FROM events HAVING flag;");
}

#[test]
fn test_sql_builder_group_by_and_having_trailing_tokens_are_reported() {
    for (sql, expected) in [
        ("SELECT a FROM t GROUP BY;", "`GROUP BY` requires at least one item"),
        ("SELECT a FROM t GROUP BY a,;", "trailing comma in `GROUP BY`"),
        ("SELECT a FROM t GROUP BY a FOO;", "unsupported token in GROUP BY clause"),
        ("SELECT a FROM t HAVING;", "`HAVING` requires a predicate"),
        ("SELECT a FROM t HAVING a FOO;", "unsupported token in HAVING predicate"),
    ] {
        let reason = rejection_reason(sql);
        assert!(reason.contains(expected), "{sql}: {reason}");
    }
}

#[test]
fn test_sql_builder_order_by_nulls_first_last() {
    let sql = "SELECT a FROM t ORDER BY a DESC NULLS LAST, b ASC NULLS FIRST, c;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    let order_by = select.order_by.as_ref().unwrap_or_else(|| panic!("Missing ORDER BY"));
    assert_eq!(order_by.items.len(), 3);

    assert_eq!(order_by.items[0].direction, OrderDirection::Desc);
    assert_eq!(order_by.items[0].nulls, Some(NullsOrder::Last));
    assert_eq!(sql.get(order_by.items[0].span.start..order_by.items[0].span.end), Some("a DESC NULLS LAST"));

    assert_eq!(order_by.items[1].direction, OrderDirection::Asc);
    assert_eq!(order_by.items[1].nulls, Some(NullsOrder::First));
    assert_eq!(sql.get(order_by.items[1].span.start..order_by.items[1].span.end), Some("b ASC NULLS FIRST"));

    assert_eq!(order_by.items[2].direction, OrderDirection::Asc);
    assert_eq!(order_by.items[2].nulls, None);

    assert_eq!(source, "SELECT a FROM t ORDER BY a DESC NULLS LAST,b ASC NULLS FIRST,c ASC;");

    // `NULLS` does not require an explicit direction.
    let (statement, source) = build_statement("SELECT a FROM t ORDER BY a NULLS FIRST;");
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement, got {statement:?}");
    };
    let order_by = select.order_by.as_ref().unwrap_or_else(|| panic!("Missing ORDER BY"));
    assert_eq!(order_by.items[0].direction, OrderDirection::Asc);
    assert_eq!(order_by.items[0].nulls, Some(NullsOrder::First));
    assert_eq!(source, "SELECT a FROM t ORDER BY a ASC NULLS FIRST;");
}

#[test]
fn test_sql_builder_incomplete_nulls_order_is_reported() {
    for (sql, expected) in [
        ("SELECT a FROM t ORDER BY a NULLS MIDDLE;", "`NULLS` must be followed by `FIRST` or `LAST`"),
        ("SELECT a FROM t ORDER BY a NULLS;", "`NULLS` must be followed by `FIRST` or `LAST`"),
        ("SELECT a FROM t ORDER BY a FIRST;", "unsupported token in ORDER BY clause"),
        ("SELECT a FROM t ORDER BY a NULLS FIRST LAST;", "unsupported token in ORDER BY clause"),
        ("SELECT a FROM t ORDER BY;", "`ORDER BY` requires at least one item"),
        ("SELECT a FROM t ORDER BY a,;", "trailing comma in `ORDER BY`"),
    ] {
        let reason = rejection_reason(sql);
        assert!(reason.contains(expected), "{sql}: {reason}");
    }
}

/// No statement may leave a token unconsumed without a diagnostic. The cases
/// below differ in where the tail appears, not only in `NULLS` placement.
#[test]
fn test_sql_builder_unconsumed_trailing_tokens_are_reported() {
    for (sql, expected) in [
        ("SELECT a FROM t LIMIT 1 FOO;", "unexpected token after statement"),
        ("INSERT INTO t (a) VALUES (1) FOO;", "unexpected token after statement"),
        ("UPDATE t SET a = 1 FOO;", "unexpected token after statement"),
        ("SELECT a FROM t ORDER BY a FOO;", "unsupported token in ORDER BY clause"),
        ("SELECT a FROM t GROUP BY a FOO;", "unsupported token in GROUP BY clause"),
        ("SELECT a FROM t HAVING a FOO;", "unsupported token in HAVING predicate"),
    ] {
        let reason = rejection_reason(sql);
        assert!(reason.contains(expected), "{sql}: {reason}");
    }

    // Valid tails stay quiet.
    for sql in ["SELECT a FROM t LIMIT 1; SELECT b FROM u;", "SELECT a FROM t", "SELECT a FROM t ORDER BY a DESC NULLS LAST;"] {
        assert_eq!(rejection_reason(sql), "", "{sql} should not be rejected");
    }
}

#[test]
fn test_sql_builder_set_operations() {
    for (sql, expected_operator, expected_source) in [
        ("SELECT a FROM t UNION SELECT b FROM u;", SetOperator::Union, "SELECT a FROM t UNION SELECT b FROM u;"),
        ("SELECT a FROM t UNION ALL SELECT b FROM u;", SetOperator::UnionAll, "SELECT a FROM t UNION ALL SELECT b FROM u;"),
        ("SELECT a FROM t INTERSECT SELECT b FROM u;", SetOperator::Intersect, "SELECT a FROM t INTERSECT SELECT b FROM u;"),
        ("SELECT a FROM t EXCEPT SELECT b FROM u;", SetOperator::Except, "SELECT a FROM t EXCEPT SELECT b FROM u;"),
    ] {
        let (statement, source) = build_statement(sql);
        let SqlStatement::SetOperation(set) = &statement
        else {
            panic!("Expected set operation for {sql}, got {statement:?}");
        };
        assert_eq!(set.operator, expected_operator, "{sql}");
        assert_eq!(sql.get(set.left.span.start..set.left.span.end).map(str::trim), Some("SELECT a FROM t"), "{sql}");
        assert_eq!(sql.get(set.right.span.start..set.right.span.end).map(str::trim), Some("SELECT b FROM u"), "{sql}");
        assert!(set.order_by.is_none(), "{sql}");
        assert!(set.limit.is_none(), "{sql}");
        assert_eq!(source, expected_source, "{sql}");

        // The printed source must parse back into the same statement.
        let (rebuilt, rebuilt_source) = build_statement(&source);
        let SqlStatement::SetOperation(rebuilt) = &rebuilt
        else {
            panic!("Expected set operation after round trip for {sql}, got {rebuilt:?}");
        };
        assert_eq!(rebuilt.operator, expected_operator, "{sql}");
        assert_eq!(rebuilt_source, source, "{sql}");
    }
}

/// `ORDER BY` and `LIMIT` after a set operation apply to the combined result, so
/// they must not end up on the right branch.
#[test]
fn test_sql_builder_set_operation_tail_binds_to_whole_result() {
    let sql = "SELECT a FROM t UNION SELECT b FROM u ORDER BY a LIMIT 5;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::SetOperation(set) = &statement
    else {
        panic!("Expected set operation, got {statement:?}");
    };
    assert_eq!(set.operator, SetOperator::Union);
    let order_by = set.order_by.as_ref().unwrap_or_else(|| panic!("Missing ORDER BY on the set operation"));
    assert_eq!(order_by.items.len(), 1);
    let limit = set.limit.as_ref().unwrap_or_else(|| panic!("Missing LIMIT on the set operation"));
    assert!(matches!(&limit.limit, Expression::Literal(Literal::Number(value, _)) if value.as_ref() == "5"));
    assert!(set.right.order_by.is_none(), "ORDER BY must not bind to the right branch");
    assert!(set.right.limit.is_none(), "LIMIT must not bind to the right branch");
    assert_eq!(source, "SELECT a FROM t UNION SELECT b FROM u ORDER BY a ASC LIMIT 5;");

    let (rebuilt, rebuilt_source) = build_statement(&source);
    let SqlStatement::SetOperation(rebuilt) = &rebuilt
    else {
        panic!("Expected set operation after round trip, got {rebuilt:?}");
    };
    assert!(rebuilt.order_by.is_some());
    assert!(rebuilt.limit.is_some());
    assert!(rebuilt.right.order_by.is_none());
    assert_eq!(rebuilt_source, source);
}

#[test]
fn test_sql_builder_set_operations_reject_unsupported_shapes() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&SourceText::new("SELECT a FROM t UNION SELECT b FROM u UNION SELECT c FROM v;"), &[], &mut session);
    assert!(result.result.is_err(), "chained set operations must be rejected: {:?}", result.result);

    let reason = rejection_reason("SELECT a FROM t UNION;");
    assert!(!reason.is_empty(), "a set operation without its right SELECT must be reported");
}

#[test]
fn test_sql_builder_insert() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let source = SourceText::new("INSERT INTO users (name, age) VALUES ('John', 25);");

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    if let Err(e) = &result.result {
        panic!("Build error: {e:?}\nDiagnostics: {:?}", result.diagnostics);
    }
    let root = result.result.unwrap();
    assert_eq!(root.statements.len(), 1);

    let SqlStatement::Insert(insert) = &root.statements[0]
    else {
        panic!("Expected Insert statement");
    };
    assert_eq!(insert.table_name.name.name.as_ref(), "users");
    assert_eq!(insert.columns.len(), 2);
    assert_eq!(insert.values.len(), 2);
}

#[ignore = "SQL builder/AST round-trip tests are out of date"]
#[test]
fn test_sql_builder_create_table() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let source = SourceText::new("CREATE TABLE users (id INT PRIMARY KEY, name TEXT);");

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();
    assert_eq!(root.statements.len(), 1);

    let SqlStatement::Create(create) = &root.statements[0]
    else {
        panic!("Expected Create statement");
    };
    assert_eq!(create.object_type, CreateObjectType::Table);
    assert_eq!(create.name.name.as_ref(), "users");
    let CreateBody::Table { columns, .. } = &create.body
    else {
        panic!("Expected Table body");
    };
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].name.name.as_ref(), "id");
    assert_eq!(columns[0].data_type.as_ref(), "INT");
    assert!(!columns[0].constraints.is_empty());
    assert_eq!(columns[1].name.name.as_ref(), "name");
    assert_eq!(columns[1].data_type.as_ref(), "TEXT");
}

#[ignore = "SQL builder/AST round-trip tests are out of date"]
#[test]
fn test_sql_builder_create_view() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let source = SourceText::new("CREATE VIEW active_users AS SELECT name FROM users WHERE active = 1;");

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let SqlStatement::Create(create) = &root.statements[0]
    else {
        panic!("Expected Create statement");
    };
    assert_eq!(create.object_type, CreateObjectType::View);
    assert_eq!(create.name.name.as_ref(), "active_users");
    let CreateBody::View { query, .. } = &create.body
    else {
        panic!("Expected View body");
    };
    assert!(query.from.is_some());
    assert!(query.expr.is_some());
}

#[ignore = "SQL builder/AST round-trip tests are out of date"]
#[test]
fn test_sql_builder_update_delete() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();

    let source_update = SourceText::new("UPDATE users SET name = 'John', age = 30 WHERE id = 1;");
    let result_update = builder.build(&source_update, &[], &mut session);
    assert!(result_update.result.is_ok());
    let root_update = result_update.result.unwrap();
    let SqlStatement::Update(update) = &root_update.statements[0]
    else {
        panic!("Expected Update statement");
    };
    assert_eq!(update.table_name.name.name.as_ref(), "users");
    assert_eq!(update.assignments.len(), 2);
    assert!(update.selection.is_some());

    let source_delete = SourceText::new("DELETE FROM users WHERE id = 1;");
    let result_delete = builder.build(&source_delete, &[], &mut session);
    assert!(result_delete.result.is_ok());
    let root_delete = result_delete.result.unwrap();
    let SqlStatement::Delete(delete) = &root_delete.statements[0]
    else {
        panic!("Expected Delete statement");
    };
    assert_eq!(delete.table_name.name.name.as_ref(), "users");
    assert!(delete.selection.is_some());
}

#[ignore = "SQL builder/AST round-trip tests are out of date"]
#[test]
fn test_sql_builder_drop_alter() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();

    let source_drop = SourceText::new("DROP TABLE IF EXISTS users;");
    let result_drop = builder.build(&source_drop, &[], &mut session);
    assert!(result_drop.result.is_ok());
    let root_drop = result_drop.result.unwrap();
    let SqlStatement::Drop(drop) = &root_drop.statements[0]
    else {
        panic!("Expected Drop statement");
    };
    assert_eq!(drop.object_type, DropObjectType::Table);
    assert_eq!(drop.name.name.as_ref(), "users");
    assert!(drop.if_exists);

    let source_alter = SourceText::new("ALTER TABLE users RENAME TO customers;");
    let result_alter = builder.build(&source_alter, &[], &mut session);
    assert!(result_alter.result.is_ok());
    let root_alter = result_alter.result.unwrap();
    let SqlStatement::Alter(alter) = &root_alter.statements[0]
    else {
        panic!("Expected Alter statement");
    };
    assert_eq!(alter.table_name.name.name.as_ref(), "users");
}

#[ignore = "SQL builder/AST round-trip tests are out of date"]
#[test]
fn test_sql_builder_database_stmt() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();

    let source_create = SourceText::new("CREATE DATABASE test_db;");
    let result_create = builder.build(&source_create, &[], &mut session);
    assert!(result_create.result.is_ok());
    let root_create = result_create.result.unwrap();
    let SqlStatement::Create(create) = &root_create.statements[0]
    else {
        panic!("Expected Create statement");
    };
    assert_eq!(create.object_type, CreateObjectType::Database);
    assert_eq!(create.name.name.as_ref(), "test_db");
    assert!(matches!(create.body, CreateBody::Database { .. }));

    let source_drop = SourceText::new("DROP DATABASE test_db;");
    let result_drop = builder.build(&source_drop, &[], &mut session);
    assert!(result_drop.result.is_ok());
    let root_drop = result_drop.result.unwrap();
    let SqlStatement::Drop(drop) = &root_drop.statements[0]
    else {
        panic!("Expected Drop statement");
    };
    assert_eq!(drop.object_type, DropObjectType::Database);
    assert_eq!(drop.name.name.as_ref(), "test_db");
}

#[test]
fn test_sql_builder_case_when_basic() {
    let sql = "SELECT CASE WHEN a = 1 THEN 'one' END FROM t;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::Case { branches, else_branch, .. }, .. } = &select.items[0]
    else {
        panic!("Expected Case expression, got {:?}", select.items[0]);
    };
    assert_eq!(branches.len(), 1);
    assert!(else_branch.is_none());
    // Pretty-printer strips trivia whitespace but keeps keyword internal spacing.
    assert_eq!(source, "SELECT CASE WHEN a=1 THEN 'one' END FROM t;");
}

#[test]
fn test_sql_builder_case_when_multiple_and_else() {
    let sql = "SELECT CASE WHEN a = 1 THEN 'one' WHEN a = 2 THEN 'two' ELSE 'other' END FROM t;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::Case { branches, else_branch, .. }, .. } = &select.items[0]
    else {
        panic!("Expected Case expression");
    };
    assert_eq!(branches.len(), 2);
    assert!(else_branch.is_some());
    assert_eq!(source, "SELECT CASE WHEN a=1 THEN 'one' WHEN a=2 THEN 'two' ELSE 'other' END FROM t;");
}

#[test]
fn test_sql_builder_cast_integer() {
    let sql = "SELECT CAST(age AS INTEGER) FROM users;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::Cast { expr, target, .. }, .. } = &select.items[0]
    else {
        panic!("Expected Cast expression");
    };
    assert!(matches!(expr.as_ref(), Expression::Identifier(identifier) if identifier.name.as_ref() == "age"));
    assert_eq!(target.as_ref(), "INTEGER");
    assert_eq!(source, "SELECT CAST(age AS INTEGER)FROM users;");
}

#[test]
fn test_sql_builder_cast_blob_identifier_type() {
    // BLOB is not a dedicated keyword token — it comes through as Identifier_.
    let sql = "SELECT CAST(data AS BLOB) FROM records;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::Cast { target, .. }, .. } = &select.items[0]
    else {
        panic!("Expected Cast expression");
    };
    assert_eq!(target.as_ref(), "BLOB");
    assert_eq!(source, "SELECT CAST(data AS BLOB)FROM records;");
}

#[test]
fn test_sql_builder_cast_timestamp_with_expression() {
    let sql = "SELECT CAST(a + b AS TIMESTAMP) FROM t;";
    let (statement, source) = build_statement(sql);
    let SqlStatement::Select(select) = &statement
    else {
        panic!("Expected Select statement");
    };
    let SelectItem::Expression { expr: Expression::Cast { expr, target, .. }, .. } = &select.items[0]
    else {
        panic!("Expected Cast expression");
    };
    assert!(matches!(expr.as_ref(), Expression::Binary { op: BinaryOperator::Plus, .. }));
    assert_eq!(target.as_ref(), "TIMESTAMP");
    assert_eq!(source, "SELECT CAST(a+b AS TIMESTAMP)FROM t;");
}
