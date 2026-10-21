# 🌳 Oaks: The Oak Language Framework Facade

The main facade for the Oak language framework.

This crate re-exports the most commonly used types and traits from the
various sub-crates in the Oak ecosystem, providing a single entry point
for building language services and tools.

`oaks` is the main entry point and facade for the Oak language framework. it provides a unified and simplified API by re-exporting the most commonly used components from across the Oak ecosystem.

## 🚀 Overview

The Oak framework is designed for building high-performance language services (like LSPs) and compilers. This crate aggregates the following capabilities:

- **Core Parsing**: Re-exports from `oak-core` for syntax tree management.
- **Virtual File System**: Integrated VFS from `oak-vfs` for file management.
- **Language Services**: High-level providers for IDE features like Hover, Folding, Symbols, and Semantic Tokens.
- **LSP Integration**: Ready-to-use types and servers for the Language Server Protocol.

## 🚦 Quick Start

Using `oaks` to build a simple language tool:

```rust
use oaks::{Language, Parser, Vfs, MemoryVfs, LanguageService};

// 1. Setup VFS
let vfs = MemoryVfs::new();
vfs.set_file_content("test.oak", "fn main() { }");

// 2. Access language features through the facade
let service = LanguageService::new(Arc::new(vfs));
let hover_info = service.hover("test.oak", Position::new(0, 5));
```

## 🏗️ Re-exported Modules

`oaks` re-exports key components from these crates:

- **[oak-core](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-core)**: The foundation of the framework (Trees, Lexer, Parser).
- **[oak-vfs](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-vfs)**: Virtual File System and line mapping.
- **[oak-lsp](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-lsp)**: LSP server and protocol types.
- **[oak-hover](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-hover)**: Hover information provider.
- **[oak-folding](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-folding)**: Folding range calculation.
- **[oak-symbols](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-symbols)**: Document and workspace symbols.

## 🛠️ Architecture

`oaks` follows the "Facade Pattern" to hide the complexity of the underlying modular system while still allowing advanced users to access specific crates directly if needed. It ensures that the most common workflows are ergonomic and well-documented.

