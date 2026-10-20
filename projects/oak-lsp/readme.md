# 🚀 oak-lsp

[![Crates.io](https://img.shields.io/crates/v/oak-lsp.svg)](https://crates.io/crates/oak-lsp)
[![Documentation](https://docs.rs/oak-lsp/badge.svg)](https://docs.rs/oak-lsp)

**Language Server Protocol Implementation for Oak** — A complete LSP server framework for building language-aware editors and IDEs.

## 🎯 Why oak-lsp?

The Language Server Protocol (LSP) has become the standard for language tooling integration. `oak-lsp` provides a complete, production-ready LSP server implementation that works with any Oak language parser.

## ✨ Key Features

- **📡 Full LSP Support** — Core LSP requests and notifications
- **🔧 Language Service Trait** — Unified interface for language-specific features
- **📂 Workspace Management** — Multi-file projects with change tracking
- **🔄 Incremental Updates** — Efficient handling of text document changes
- **🛡️ Error Recovery** — Robust error handling for malformed requests

## 🏗️ Architecture

| Module | Purpose |
|--------|---------|
| `handlers` | LSP request/notification handlers |
| `service` | `LanguageService` trait |
| `types` | LSP-specific type definitions |
| `workspace` | File and project management |

### Supported LSP Features

- `textDocument/completion` — Code completion
- `textDocument/hover` — Hover information
- `textDocument/definition` — Go to definition
- `textDocument/references` — Find all references
- `textDocument/documentSymbol` — Document outline
- `textDocument/foldingRange` — Code folding
- `textDocument/semanticTokens` — Semantic highlighting
- `textDocument/diagnostic` — Diagnostics

## 🔗 Ecosystem Integration

Integrates with `oak-vfs` for file access, `oak-hover`/`oak-navigation`/`oak-symbols` for features, and any LSP-compatible editor.

## 📖 Documentation

For usage examples and API details, see the [API documentation](https://docs.rs/oak-lsp).

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
