# TypeScript/JavaScript formatter

Public module: `oak_typescript::formatter` (CST rules live under `formatter/`, not a separate `cst_format` tree).

- [`FormatOptions`](options.rs) — `indent_width`, `line_width`
- [`FormatError`](error.rs) — formatting failure message
- [`format_source`](engine.rs) — `source` in, formatted `String` out

Shared product contract tests live in downstream consumers (`nifty-formatter`, `vmz-formatter`), not in this crate.
