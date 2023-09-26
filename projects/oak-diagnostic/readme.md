# `oak-diagnostic` (deprecated)

This crate is a **transitional facade** and will be removed.

- Unified Oak parser diagnostics: `oak_core::diagnostic`
- Shared diagnostic model and terminal rendering: `diagnostic` (`rust-boost`)
- Legacy `ConsoleEmitter` / `HtmlEmitter` types in this crate are deprecated

New code must not depend on `oak-diagnostic`. Migrate to `oak-core` with the `diagnostic` feature.
