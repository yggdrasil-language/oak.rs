# 🛠️ SQL Parser Developer Guide

SQL support for the Oak language framework.

This guide is designed to help you quickly get started with developing and integrating `oak-sql`.

## 🚦 Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
oak-sql = { path = "..." }
```

### Basic Parsing Example

The following is a standard workflow for parsing a complex SQL query with Joins and CTEs:

```rust
use oak_core::{SourceText, parser::{Parser, session::ParseSession}};
use oak_sql::{SqlLanguage, SqlParser};

fn main() {
    // 1. Prepare source code (PostgreSQL dialect)
    let code = r#"
        WITH regional_sales AS (
            SELECT region, SUM(amount) AS total_sales
            FROM orders
            GROUP BY region
        )
        SELECT product_name, total_sales
        FROM regional_sales
        JOIN products ON products.region = regional_sales.region
        WHERE total_sales > 10000
        ORDER BY total_sales DESC;
    "#;
    let source = SourceText::new(code);

    // 2. Initialize parser with specific dialect configuration
    let config = SqlLanguage::postgresql();
    let parser = SqlParser::new(&config);
    let mut session: ParseSession<_> = ParseSession::default();

    // 3. Execute parsing
    let result = parser.parse(&source, &[], &mut session);

    // 4. Handle results
    if !result.has_errors() {
        println!("Parsing successful!");
    } else {
        eprintln!("Errors found during parsing.");
    }
}
```

## 🔍 Core API Usage

### 1. Syntax Tree Traversal
After a successful parse, you can use the built-in visitor pattern or manually traverse the Green/Red Tree to extract SQL-specific constructs like CTE definitions, Join conditions, `WHERE` clauses, or DDL table structures.

### 2. Incremental Parsing
No need to re-parse an entire multi-megabyte SQL dump when small changes occur:
```rust
use oak_core::{SourceText, TextEdit, parser::{Parser, session::ParseSession}};
use oak_sql::{SqlLanguage, SqlParser};

let config = SqlLanguage::postgresql();
let parser = SqlParser::new(&config);

let old_source = SourceText::new("SELECT a FROM t WHERE x = 1".to_string());
let mut session: ParseSession<_> = ParseSession::default();
let _old_result = parser.parse(&old_source, &[], &mut session);

let new_source = SourceText::new("SELECT a, b FROM t WHERE x = 1".to_string());
// Session carries the previous tree and enables node reuse via edits.
let edits = [TextEdit { span: (7usize..7).into(), text: ", b".into() }];
let new_result = parser.parse(&new_source, &edits, &mut session);
# let _ = new_result;
```

### 3. Diagnostics
`oak-sql` provides rich error contexts specifically tailored for database developers, handling dialect-specific syntax quirks and providing clear feedback on malformed queries:
```rust
use oak_core::{SourceText, parser::{Parser, session::ParseSession}};
use oak_sql::{SqlLanguage, SqlParser};

let config = SqlLanguage::postgresql();
let parser = SqlParser::new(&config);
let source = SourceText::new("SELECT a FROM t WHERE".to_string());
let mut session: ParseSession<_> = ParseSession::default();
let result = parser.parse(&source, &[], &mut session);

for diag in &result.diagnostics {
    if let Some(offset) = diag.source_offset() {
        println!("[offset {}] {}", offset, diag);
    } else {
        println!("{}", diag);
    }
}
```

## 🏗️ Architecture Overview

- **Lexer**: Tokenizes SQL source text into a stream of tokens, handling keywords, operators, quoted identifiers, and dialect-specific literals.
- **Parser**: Syntax analyzer based on the Pratt parsing algorithm to handle SQL's complex expression precedence, multi-dialect support, and deeply nested query structures.
- **AST**: A strongly-typed syntax abstraction layer designed for high-performance SQL analysis tools, query formatters, and IDEs.

## 🔗 Advanced Resources

- **Full Examples**: Check the [examples/](examples/) folder in the project root.
- **API Documentation**: Run `cargo doc --open` for detailed type definitions.
- **Test Cases**: See [tests/](tests/) for handling of various SQL dialects and edge cases.
