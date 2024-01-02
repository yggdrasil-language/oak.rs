# Vue SFC `format` API

Public module: `oak_vue::formatter`.

- [`FormatOptions`](options.rs) — `indent_width`, `line_width`
- [`FormatError`](error.rs) — shared Oak diagnostics with original source offsets
- [`format_source`](engine.rs) — parse validation and byte-preserving output (transition)

CST layout rules are not implemented yet. This entry does not use a pretty-print document or normalize source trivia.
