# TypeScript/JavaScript `format` API

`oak-typescript` exposes one product operation for formatting source text:

- [`FormatOptions`](format/options.rs) — `indent_width`, `line_width`
- [`FormatError`](format/error.rs) — formatting failure message
- [`format_source`](format/engine.rs) — `source` in, formatted `String` out

Tree-aware and AST-based implementation modules are **private**. Consumers must not
depend on internal module paths.

## Contract (current)

| Behavior | Status |
| --- | --- |
| Supported top-level statements | Partial (const/import/export/expression) |
| Leading/trailing comments & ASI-sensitive layout | Partial (preserve when detected) |
| Invalid/incomplete source | Must `Err` without silent rewrite |
| Idempotence | Required for supported inputs |
| JSX | Via transitional AST output fallback |

Matrix and fixtures: VMZ handoff `2026-10-02-oak-formatter-capability-matrix.md`.
