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

    let Expression::Binary { op: BinaryOperator::Equal, right, .. } = build_where("SELECT * FROM t WHERE name IS NULL;")
    else {
        panic!("Expected IS NULL");
    };
    assert!(matches!(right.as_ref(), Expression::Literal(Literal::Null(_))));

    let Expression::Binary { op: BinaryOperator::NotEqual, right, .. } = build_where("SELECT * FROM t WHERE name IS NOT NULL;")
    else {
        panic!("Expected IS NOT NULL");
    };
    assert!(matches!(right.as_ref(), Expression::Literal(Literal::Null(_))));

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
    assert!(args.is_empty(), "COUNT(*) must not invent an argument: {args:?}");

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
