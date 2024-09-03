# 🛠️ TeX/LaTeX Parser Developer Guide

TeX support for the Oak language framework.

This crate provides lexing, parsing, and AST building for TeX/LaTeX documents,
enabling integration with the Oak language server and other tools.

This guide is designed to help you quickly get started with developing and integrating `oak-tex`.

## 🚦 Quick Start

Add the dependency to your `Cargo.toml`:

```toml
[dependencies]
oak-tex = { path = "..." }
```

### Basic Parsing Example

The following is a standard workflow for parsing TeX/LaTeX documents, supporting commands, environments, and math mode:

```rust
use oak_core::{Builder, SourceText, parser::session::ParseSession};
use oak_tex::{TexBuilder, TexLanguage};

fn main() {
    // 1. Prepare source code
    let code = r#"
        \documentclass{article}
        \begin{document}
        The value of $\pi$ is approximately 3.14159.
        \end{document}
    "#;
    let source = SourceText::new(code);

    // 2. Initialize the language and the AST builder
    let language = TexLanguage::default();
    let builder = TexBuilder::new(&language);

    // 3. Build the typed AST
    let mut session = ParseSession::<TexLanguage>::default();
    let output = builder.build(&source, &[], &mut session);

    // 4. Handle results
    match output.result {
        Ok(root) => println!("Parsing successful! Top-level items: {}", root.items.len()),
        Err(error) => eprintln!("Errors found during parsing: {error}"),
    }
    for diagnostic in &output.diagnostics {
        println!("diagnostic: {diagnostic}");
    }
}
```

## 🔍 Core API Usage

### 1. Syntax Tree Traversal
After a successful parse, you can use the built-in visitor pattern or manually traverse the Green/Red Tree to extract TeX specific constructs like commands, environments, math mode groups, and plain text content.

### 2. Incremental Parsing
TeX documents can be very large (e.g., theses, books). `oak-tex` reuses the session cache across builds, so unchanged subtrees survive incremental updates:
```rust
# use oak_core::{Builder, SourceText, parser::session::ParseSession};
# use oak_tex::{TexBuilder, TexLanguage};
# let language = TexLanguage::default();
# let builder = TexBuilder::new(&language);
# let mut session = ParseSession::<TexLanguage>::default();
# let old_source = SourceText::new(r#"\section{Old title}"#);
# let old_output = builder.build(&old_source, &[], &mut session);
// Rebuild from the edited source. The same session carries the typed-node
// cache from previous builds, so unchanged subtrees can be reused.
let new_source = SourceText::new(r#"\section{New title}"#);
let new_output = builder.build(&new_source, &[], &mut session);
assert!(new_output.result.is_ok());
```

### 3. Error Recovery
The parser is designed for industrial-grade fault tolerance, recovering gracefully from unclosed environments or missing braces to provide continuous feedback in IDEs.

## 🏗️ Architecture Overview

- **Lexer**: Tokenizes TeX source text, supporting control sequences, category codes (catcodes), and special characters.
- **Parser**: A high-performance recursive descent parser that handles TeX's macro-based expansion and environment nesting.
- **AST**: A strongly-typed, lossless syntax tree that preserves all trivia (comments/whitespace) for refactoring and formatting tools.

## 🔗 Advanced Resources

- **Full Examples**: Check the [examples/](examples/) folder in the project root.
- **API Documentation**: Run `cargo doc --open` for detailed type definitions.
- **Test Cases**: See [tests/readme.md](tests/readme.md) for details on our snapshot-based testing.
