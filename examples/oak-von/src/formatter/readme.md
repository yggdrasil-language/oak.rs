# VON formatter

Public module: `oak_von::formatter` (CST rules live in private `formatter/` submodules).

- [`FormatOptions`](options.rs) — `indent_width`, `line_width`
- [`FormatError`](error.rs) — formatting failure message
- [`format_source`](engine.rs) — `source` in, formatted `String` out

Pipeline: lossless `VonLexer` → token-gap rules via `oak_formatter` → `apply_edits`.
