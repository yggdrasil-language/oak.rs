# Oaks - Parser Framework for Rust

[![Rust Version](https://img.shields.io/badge/rust-nightly-blue.svg)](https://www.rust-lang.org)

Oaks is a modular parser framework for Rust that provides a unified approach to building language parsers. Built on the `oak-core` foundation, Oaks offers a comprehensive set of tools for lexical analysis, parsing, and syntax tree manipulation.

## 🌟 Why Oaks?

Oaks is designed for developers who need more than what parser generators can offer. Our philosophy focuses on three core pillars:

- **Hand-written Optimization**: Unlike generic parser generators, Oaks encourages hand-written parsing logic. This gives you absolute control over the parsing process, allowing for micro-optimizations that generators often miss.
- **Language-Specific Acceleration**: Every language has its own quirks. Oaks provides the infrastructure to leverage language-specific features for maximum performance, ensuring that your parser is as fast as possible for its specific domain.
- **Human-Centric Design**: We believe parsers should be easy to debug and use. Oaks produces highly readable syntax trees and provides clear, actionable error messages that help developers identify and fix issues instantly.

## 🚀 Features

- **Modular Architecture**: Decouples the core parsing engine from language-specific logic. Implement new languages by defining `TokenType` and `ElementType` without touching the core infrastructure.
- **Lossless Green/Red Tree**: Implements a Rowan-style architecture. **Green Trees** are immutable and interned for memory efficiency, while **Red Trees** provide a parent-aware, position-aware view for effortless traversal.
- **Structural Sharing**: Modifications to the tree use `Arc`-based sharing. Only the modified nodes and their direct ancestors are recreated, making transformations and refactorings extremely memory-efficient.
- **Error Recovery**: The parser can recover from syntax errors to produce a partial but valid tree, ensuring that features like highlighting and autocompletion remain functional during active editing.
- **Incremental Parsing**: By utilizing an `IncrementalCache`, the framework only re-parses the changed portions of the source code, enabling lightning-fast updates for large files in IDE environments.
- **Source Mapping**: Absolute position information is dynamically computed from Red Nodes, providing a stable and accurate foundation for Source Maps and diagnostic reporting.

## 🛠️ Core Capabilities

### Formatter
Implemented by manipulating `Trivia` (whitespace, line breaks, comments) within the `GreenNode` structure. It ensures 100% lossless code formatting by reconstructing the source text while preserving or adjusting non-functional tokens.

### Linter
Utilizes the `Visitor` pattern to traverse the `RedTree`. By leveraging absolute `span` information computed from red nodes, it performs efficient static analysis and provides precise diagnostic locations for coding standard violations.

### Highlighter
Supports dual-mode highlighting: fast Lexer-based highlighting using the raw token stream, and precise Parser-based highlighting that utilizes the full syntax tree to distinguish between semantic categories like function calls, types, and variables.

### Transformer
Powered by the `Transformer` trait, it enables high-performance code refactoring. It leverages the **Structural Sharing** property of the Red-Green tree architecture; only modified nodes and their parent paths are recreated as new `GreenNodes`, while unchanged subtrees are efficiently reused via `Arc`.

## 📦 Core Components

| Component          | Description                           | Status         |
|--------------------|---------------------------------------|----------------|
| `oak-core`         | Core parsing infrastructure and traits| ✅ Active       |
| `oak-lsp`          | Language Server Protocol support      | ✅ Active       |
| `oak-vfs`          | Virtual File System for analysis      | ✅ Active       |
| `oak-highlight`    | Multi-language syntax highlighter     | ✅ Active       |
| `oak-pretty-print` | Code formatting and pretty printing   | ✅ Active       |
| `oak-mcp`          | Model Context Protocol integration    | ✅ Active       |
| `oak-hover`        | Semantic hover information provider   | ✅ Active       |
| `oak-repl`         | Interactive parser testing tool       | ✅ Active       |
| `oak-visualize`    | AST visualization and graph rendering | 🔄 Development |
| `oaks`             | Main unified library                  | ✅ Active       |

## 🔧 Language Parsers

Oaks supports a wide range of languages through its modular architecture. Below are some of the supported parsers:

### System & Compiled Languages
- `oak-c`, `oak-cpp`, `oak-rust`, `oak-go`, `oak-zig`, `oak-swift`, `oak-ada`, `oak-d`, `oak-nim`, `oak-vlang`

### Web & Scripting
- `oak-javascript`, `oak-html`, `oak-css`, `oak-sass`, `oak-scss`, `oak-stylus`, `oak-vue`, `oak-php`, `oak-python`, `oak-ruby`, `oak-perl`, `oak-lua`, `oak-bash`, `oak-powershell`, `oak-bat`, `oak-cmd`, `oak-tcl`

### Data & Configuration
- `oak-json` (with JSON5), `oak-yaml`, `oak-toml`, `oak-ini`, `oak-csv`, `oak-xml`, `oak-nix`, `oak-dhall`, `oak-dot`

### Functional & Specialized
- `oak-java`, `oak-kotlin`, `oak-scala`, `oak-clojure`, `oak-elixir`, `oak-erlang`, `oak-fsharp`, `oak-ocaml`, `oak-julia`, `oak-matlab`, `oak-r`
- `oak-markdown`, `oak-ascii-doc`, `oak-tex`, `oak-typst`
- `oak-sql`, `oak-regex`, `oak-wgsl`, `oak-hlsl`, `oak-wat`
- `oak-coq`, `oak-lean`, `oak-prolog`

### Legacy & Industrial
- `oak-cobol`, `oak-pascal`, `oak-delphi`, `oak-vhdl`

### Internal & Experimental
- `oak-awsl`, `oak-voml`, `oak-von`, `oak-gsgl`, `oak-jasm`, `oak-msil`, `oak-mojo`

*...and many more being added regularly.*

## 🛠️ Quick Start

Basic usage example with oak-c:

```rust
use oak_core::{SourceText, Parser};
use oak_c::{CLanguage, CParser};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create source text from input
    let source = SourceText::new("int main() { return 0; }");
    
    // Parse the source code
    let parser = CParser::new(CLanguage::default());
    let result = parser.parse(&source);
    
    // Handle the result
    match result {
        Ok(tree) => println!("Parsed successfully: {:?}", tree),
        Err(errors) => println!("Parse errors: {:?}", errors),
    }
    
    Ok(())
}
```

## 🏗️ Architecture

Oaks is designed from the ground up to be a **High-Performance LSP Foundation**. Its architecture solves the most challenging problems in building modern IDE support:

### The IDE & AI Powerhouse
- **Native LSP & MCP Support**: Built-in support for Language Server Protocol and Model Context Protocol, enabling seamless integration with both modern IDEs and AI agents.
- **Virtual File System (VFS)**: Integrated `oak-vfs` provides a high-performance, incremental file system abstraction, perfect for workspace-wide analysis and cross-file refactorings.
- **Resilient Analysis**: The framework's **Error Recovery** ensures that your analysis tools remain responsive even when the code is in an invalid state.
- **HMR-Ready**: Sub-millisecond **Incremental Parsing** means your tools can provide instant feedback on every keystroke, even in multi-megabyte files.
- **Refactoring Engine**: The `Transformer` trait combined with **Structural Sharing** allows for complex code actions to be implemented with high performance and 100% comment preservation.

### Semantic Integration Ready
While Oaks focuses on high-performance syntax analysis, it is designed to be the perfect foundation for semantic analysis:
- **Semantic Hints**: The `TokenType` and `ElementType` traits provide built-in hooks like `is_definition`, `is_reference`, and `is_scope_boundary`, allowing external semantic engines to instantly understand the tree's logical structure.
- **Stable Identifiers**: Red nodes provide stable pointers that semantic analyzers can use for symbol indexing and cross-referencing.
- **Parent-Aware Navigation**: The `RedTree` allows semantic checkers to easily bubble up from a usage to its scope or declaration.
- **Typed IR**: The high-level **Typed AST** layer serves as a clean, serializable Intermediate Representation (IR) that external type-checkers and symbol solvers can consume without knowing the details of the red-green tree.
- **Framework Agnostic**: Oaks is unopinionated about how you handle semantics, making it trivial to integrate with databases (like Salsa) or graph-based analysis engines.

### Core Framework Concepts
- **Language Trait**: A unified interface to plug in any grammar.
- **Green Tree**: The "What" — immutable, shared, and extremely compact.
- **Red Tree**: The "Where" — a lightweight, parent-aware view for easy tree walking.
- **Visitor & Transformer**: Standardized patterns for both read-only analysis (Linter) and read-write mutations (Refactoring).

### Language Implementations

Each language parser (e.g., `oak-c`, `oak-json`) follows a consistent pattern:

1. **SyntaxKind Enum**: Defines all possible syntax elements
2. **Language Struct**: Implements the Language trait
3. **Lexer Struct**: Implements tokenization for the language
4. **AST Definitions**: Optional typed AST structures

### Project Structure

```
oaks/
├── projects/              # Core libraries
│   ├── oak-core/         # Core parsing infrastructure
│   ├── oak-lsp/          # LSP integration
│   ├── oak-mcp/          # MCP integration
│   ├── oak-vfs/          # Virtual File System
│   ├── oak-highlight/    # Syntax highlighting
│   ├── oak-pretty-print/ # Code formatting
│   ├── oak-hover/        # Hover support
│   ├── oak-repl/         # Interactive testing
│   ├── oak-visualize/    # AST visualization
│   └── oaks/             # Main unified library
├── examples/             # Language parsers
│   ├── oak-c/           # C parser
│   ├── oak-rust/        # Rust parser
│   ├── oak-json/        # JSON parser
│   └── ...              # 50+ other languages
└── Cargo.toml           # Workspace configuration
```

## 🔨 Build & Development

### Requirements

- Rust nightly toolchain (required for `new_range_api` feature)
- Cargo workspace support

##  Development Status

Oaks is actively developed and maintained. Current status:

### ✅ Completed

- **Core Framework (`oak-core`)**: Language trait, Lexer/Parser infrastructure, Green/Red tree system, SourceText with line/column tracking, and robust error recovery.
- **IDE & AI Infrastructure**:
    - `oak-lsp`: Full Language Server Protocol support.
    - `oak-mcp`: Model Context Protocol for AI agent integration.
    - `oak-vfs`: High-performance Virtual File System.
    - `oak-highlight`: Multi-language syntax highlighting.
    - `oak-pretty-print`: Advanced code formatting.
    - `oak-repl`: Interactive testing environment.
- **Language Support**:
    - 50+ language parsers in various stages of completion.
    - Mature implementations for C, JSON, Rust, Go, and more.
- **Performance**:
    - Incremental parsing support across all core components.
    - Zero-copy lexing and efficient structural sharing.

### 🔄 In Development

- **Advanced Semantic Analysis**: Symbol resolution, type checking, and cross-reference indexing.
- **Extended Tooling**: Enhanced `oak-visualize` and automated benchmarking suites.
- **Language Coverage**: Finalizing AST implementations for legacy and specialized languages.
- **Documentation**: Comprehensive guides and API references.

### 📋 Planned

- **Cloud-Native Deployment**: Optimized for WASM and remote development environments.
- **Universal Refactoring Engine**: Cross-language refactoring tools powered by `Transformer`.
- **Advanced AI Integration**: Deep learning-assisted error recovery and code completion.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Adding a New Language Parser

To add a new language parser to Oaks:

1. Create a new directory in `examples/` following the pattern `oak-{language}`
2. Implement the required components:
   - `SyntaxKind` enum in `src/kind/`
   - `Language` implementation in `src/language/`
   - `Lexer` implementation in `src/lexer/`
   - Optional: AST definitions in `src/ast/`
3. Add your parser to the workspace in the root `Cargo.toml`
4. Add documentation and examples

For reference implementations, see `oak-c` and `oak-json`.

## 📚 Resources

- [Repository](https://github.com/ygg-lang/oaks)
- [Issue Tracker](https://github.com/ygg-lang/oaks/issues)
- Development documentation in each crate's `readme.md`
- Examples in the `examples/` directory

---

**Happy Parsing!** 🎉
