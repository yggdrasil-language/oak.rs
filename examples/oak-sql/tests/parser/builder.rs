use oak_core::Builder;
use oak_sql::{SqlBuilder, SqlLanguage};

#[test]
fn test_select_ast() {
    let language = SqlLanguage::default();
    let builder = SqlBuilder::new(&language);
    let mut cache = oak_core::parser::session::ParseSession::default();

    let source = "SELECT id, name FROM users WHERE age > 18;";
    let result = builder.build(source, &[], &mut cache);

    if let Err(e) = &result.result {
        panic!("Build failed: {e:?}");
    }
    let root = result.result.unwrap();
    assert_eq!(root.statements.len(), 1);

    let oak_sql::ast::SqlStatement::Select(select) = &root.statements[0]
    else {
        panic!("Expected Select statement");
    };
    assert_eq!(select.items.len(), 2);
    assert!(select.from.is_some());
    assert!(select.expr.is_some());
}

#[test]
fn test_update_assignment_accepts_bare_identifier() {
    let language = SqlLanguage::default();
    let builder = SqlBuilder::new(&language);
    let mut cache = oak_core::parser::session::ParseSession::default();

    // A parameter rewriter replaces placeholders with plain identifiers, so
    // `SET column = <identifier>` must build like any other value expression.
    let source = "UPDATE accounts SET owner = next_owner WHERE id = 1";
    let result = builder.build(source, &[], &mut cache);

    if let Err(e) = &result.result {
        panic!("Build failed: {e:?}");
    }
    let root = result.result.unwrap();
    let oak_sql::ast::SqlStatement::Update(update) = &root.statements[0]
    else {
        panic!("Expected Update statement");
    };
    assert_eq!(update.assignments.len(), 1);
    assert!(matches!(update.assignments[0].value, oak_sql::ast::Expression::Identifier(_)));
    assert!(update.selection.is_some());
}

#[ignore = "SQL builder/AST round-trip tests are out of date"]
#[test]
fn test_create_table_ast() {
    let language = SqlLanguage::default();
    let builder = SqlBuilder::new(&language);
    let mut cache = oak_core::parser::session::ParseSession::default();

    let source = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL);";
    let result = builder.build(source, &[], &mut cache);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();
    assert_eq!(root.statements.len(), 1);

    let oak_sql::ast::SqlStatement::Create(create) = &root.statements[0]
    else {
        panic!("Expected Create statement");
    };
    assert_eq!(create.name.name.as_ref(), "users");
    let oak_sql::ast::CreateBody::Table { columns, .. } = &create.body
    else {
        panic!("Expected Table body");
    };
    assert_eq!(columns.len(), 2);
    assert_eq!(columns[0].name.name.as_ref(), "id");
    assert_eq!(columns[0].data_type.as_ref(), "INT");
    assert_eq!(columns[1].name.name.as_ref(), "name");
    assert_eq!(columns[1].data_type.as_ref(), "VARCHAR(255)");
}
