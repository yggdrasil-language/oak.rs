# 🛠️ DOT Parser Developer Guide

Dot support for the Oak language framework.

This guide is designed to help you quickly get started with developing and integrating `oak-dot`.

## 🚦 Quick Start

### Basic Parsing Example

The following is a standard workflow for parsing a simple DOT graph:

```rust
use oak_core::{Builder, Parser, SourceText, parser::session::ParseSession};
use oak_dot::{DotBuilder, DotLanguage, DotParser};

// 1. Prepare source code
let code = r#"
    digraph G {
        main -> parse -> execute;
        main -> init;
        main -> cleanup;
        execute -> make_string;
        execute -> printf;
        main -> printf;
        execute -> compare;
    }
"#;
let source = SourceText::new(code);

// 2. Initialize the parser and build the syntax tree
let config = DotLanguage::new();
let parser = DotParser::new(&config);
let mut cache = ParseSession::<DotLanguage>::default();
let output = parser.parse(&source, &[], &mut cache);
assert!(output.diagnostics.is_empty());

// 3. Build the typed AST
let builder = DotBuilder::new(&config);
let mut cache = ParseSession::<DotLanguage>::default();
let result = builder.build(&source, &[], &mut cache);
let root = result.result.expect("the graph should parse");
assert_eq!(root.graphs.len(), 1);
```

## 🔍 Core API Usage

### 1. Syntax Tree Traversal
After a successful parse, you can traverse the red tree to extract DOT constructs like graph definitions, node statements, edge connections, or attribute lists. The typed AST also covers them directly, see the `ast` module for `Graph`, `NodeStatement`, `EdgeStatement`, `AttributeStatement`, `SubgraphStatement` and `NodeId`.

### 2. Incremental Parsing
The parser reuses the cache it is given, so parsing an edited source again only reuses unchanged structure:
```rust
use oak_core::{Parser, SourceText};
use oak_dot::{DotLanguage, DotParser};

let config = DotLanguage::new();
let parser = DotParser::new(&config);
let mut cache = oak_core::parser::session::ParseSession::<DotLanguage>::default();

let first = SourceText::new("digraph { A -> B }");
let _ = parser.parse(&first, &[], &mut cache);

let edited = SourceText::new("digraph { A -> B; B -> C }");
let _ = parser.parse(&edited, &[], &mut cache);
```

### 3. Diagnostics
`oak-dot` recovers from malformed input and reports every skipped piece with its byte offset:
```rust
use oak_core::{Builder, OakErrorKind, SourceText, parser::session::ParseSession};
use oak_dot::{DotBuilder, DotLanguage};

let config = DotLanguage::new();
let builder = DotBuilder::new(&config);
let source = SourceText::new("digraph { A -> }");
let mut cache = ParseSession::<DotLanguage>::default();

let result = builder.build(&source, &[], &mut cache);
for diagnostic in &result.diagnostics {
    if let OakErrorKind::SyntaxError { message, offset, .. } = diagnostic.kind() {
        println!("{offset}: {message}");
    }
}
assert!(!result.diagnostics.is_empty());
```

## 🏗️ Architecture Overview

- **Lexer**: Tokenizes DOT source text into a stream of tokens, handling keywords, operators (like `->` or `--`), port separators and comments.
- **Parser**: Recursive descent over tokens, producing a lossless syntax tree with composite elements for graphs, statements, subgraphs, attribute lists, node ids and ports.
- **AST**: A strongly-typed syntax abstraction layer with byte spans on every node, designed for building high-performance graph analysis tools and visualizers.

## 🔗 Advanced Resources

- **Full Examples**: Check the [examples/](examples/) folder in the project root.
- **API Documentation**: Run `cargo doc --open` for detailed type definitions.
- **Test Cases**: See [tests/](tests/) for handling of various DOT edge cases and styling attributes.
