use oak_core::{ParseSession, SourceText, builder::Builder, source::ToSource};
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::AsDocument;
use oak_sql::{SqlBuilder, SqlLanguage};

#[test]
fn test_sql_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "SELECT name, age FROM users WHERE age > 18;";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    println!("Generated source: {}", generated);
    // ToSource adds spaces between tokens, and we expect it to be similar to input but maybe not identical in spacing
    assert!(generated.contains("SELECT"));
    assert!(generated.contains("name,age"));
    assert!(generated.contains("FROM users"));
    assert!(generated.contains("WHERE age>18"));
}

#[test]
#[cfg(feature = "oak-pretty-print")]
fn test_sql_to_doc() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "SELECT name, age FROM users WHERE age > 18;";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let doc = root.as_document(&());
    let formatted = doc.render();
    println!("Formatted doc:\n{}", formatted);

    // Pretty-printer may insert extra spaces between tokens; compare on whitespace-normalized text.
    let normalized = formatted.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains("SELECT"));
    assert!(normalized.contains("FROM users"));
    assert!(normalized.contains("WHERE age > 18"));
}

#[test]
fn test_sql_insert_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "INSERT INTO users (name, age) VALUES ('John', 25);";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    println!("Generated insert source: {}", generated);
    assert!(generated.contains("INSERT INTO users"));
    assert!(generated.contains("VALUES('John',25)"));
}

#[test]
fn test_sql_create_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "CREATE TABLE users (id INT PRIMARY KEY, name TEXT);";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    assert!(generated.contains("CREATE TABLE users"));
    assert!(generated.contains("id INT PRIMARY KEY"));
    assert!(generated.contains("name TEXT"));
}

#[test]
fn test_sql_create_view_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "CREATE VIEW user_names AS SELECT name FROM users;";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    assert!(generated.contains("CREATE VIEW user_names AS SELECT name FROM users"));
}

#[test]
fn test_sql_create_index_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "CREATE INDEX idx_user_name ON users (name);";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    assert!(generated.contains("CREATE INDEX idx_user_name ON users(name)"));
}

#[test]
fn test_sql_alter_to_source_complex_type() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "ALTER TABLE users ADD COLUMN name VARCHAR(255);";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    println!("Generated alter source: {}", generated);
    assert!(generated.contains("ALTER TABLE users"));
    assert!(generated.contains("ADD COLUMN name VARCHAR(255)"));
}

#[test]
fn test_sql_create_unique_index_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "CREATE UNIQUE INDEX idx_email ON users (email);";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let generated = root.to_source_string();
    assert!(generated.contains("CREATE UNIQUE INDEX idx_email ON users(email)"));
}

#[test]
fn test_sql_database_to_source() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let mut session = ParseSession::<SqlLanguage>::default();

    // CREATE DATABASE
    let input_create = "CREATE DATABASE test_db;";
    let source_create = SourceText::new(input_create);
    let result_create = builder.build(&source_create, &[], &mut session);
    assert!(result_create.result.is_ok());
    let generated_create = result_create.result.unwrap().to_source_string();
    assert!(generated_create.contains("CREATE DATABASE test_db"));

    // DROP DATABASE
    let input_drop = "DROP DATABASE test_db;";
    let source_drop = SourceText::new(input_drop);
    let mut session_drop = ParseSession::<SqlLanguage>::default();
    let result_drop = builder.build(&source_drop, &[], &mut session_drop);
    assert!(result_drop.result.is_ok());
    let generated_drop = result_drop.result.unwrap().to_source_string();
    assert!(generated_drop.contains("DROP DATABASE test_db"));
}

#[test]
#[cfg(feature = "oak-pretty-print")]
fn test_sql_pretty_print_complex() {
    let config = SqlLanguage::default();
    let builder = SqlBuilder::new(&config);
    let input = "CREATE TABLE users (id INT PRIMARY KEY, name VARCHAR(255) NOT NULL);";
    let source = SourceText::new(input);

    let mut session = ParseSession::<SqlLanguage>::default();
    let result = builder.build(&source, &[], &mut session);

    assert!(result.result.is_ok());
    let root = result.result.unwrap();

    let doc = root.as_document(&());
    let formatted = doc.render();
    println!("Formatted CREATE TABLE:\n{}", formatted);

    // Pretty-printer may insert extra spaces / parens spacing; compare on whitespace-normalized text.
    let normalized = formatted.split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(normalized.contains("CREATE TABLE users"));
    assert!(normalized.contains("id INT PRIMARY KEY"));
    assert!(normalized.contains("name VARCHAR ( 255 ) NOT NULL") || normalized.contains("name VARCHAR(255) NOT NULL"));
}
