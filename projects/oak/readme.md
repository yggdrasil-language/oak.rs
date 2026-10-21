# 🚀 oaks

[![Crates.io](https://img.shields.io/crates/v/oaks.svg)](https://crates.io/crates/oaks)
[![Documentation](https://docs.rs/oaks/badge.svg)](https://docs.rs/oaks)

**The Unified Oak Library** — A single crate that re-exports all Oak components for convenient access.

## 🎯 Why oaks?

While Oak is designed as a modular ecosystem, sometimes you need everything in one place. `oaks` is the unified library that re-exports all core Oak crates, providing a single dependency for the complete Oak experience.

## ✨ Key Features

- **📦 Single Dependency** — One crate for all Oak components
- **🔄 Re-exports** — Direct access to all modules
- **🎯 Convenience** — No need to manage multiple crate versions
- **🧩 Feature Flags** — Optional features for smaller compile times

## 🏗️ Architecture

### Re-exported Crates

| Crate | Purpose |
|-------|---------|
| `oak-core` | Core parsing infrastructure |
| `oak-lsp` | Language Server Protocol support |
| `oak-vfs` | Virtual File System |
| `oak-hover` | Hover information providers |
| `oak-navigation` | Code navigation features |
| `oak-folding` | Code folding support |
| `oak-symbols` | Symbol management |
| `oak-semantic-tokens` | Semantic highlighting |
| `oak-structural-view` | Document structure views |

## 🔗 When to Use

Use `oaks` when you need the complete Oak toolkit, simplified dependency management, or quick prototyping.

Use individual crates when you need minimal dependencies, faster compile times, or specific functionality only.

## 📖 Documentation

For usage examples and API details, see the [API documentation](https://docs.rs/oaks).

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
