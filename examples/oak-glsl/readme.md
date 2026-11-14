# 🚀 oak-glsl

[![Crates.io](https://img.shields.io/crates/v/oak-glsl.svg)](https://crates.io/crates/oak-glsl)
[![Documentation](https://docs.rs/oak-glsl/badge.svg)](https://docs.rs/oak-glsl)

**Shading across OpenGL and Vulkan** — An incremental GLSL parser built on the Oak framework.

## ✨ Core Features

- **⚡ Blazing Fast**: Native Rust parsing with sub-millisecond responses for typical shader sources.
- **🔄 Incremental Parsing**: Re-parse only the edited region of a shader through the Oak green tree.
- **🌳 Typed AST**: Functions, structs, globals, `layout` qualifiers, interface blocks, statements, and expressions.
- **🛡️ Error Recovery**: Truncated or malformed shaders still produce a usable tree with span diagnostics.
