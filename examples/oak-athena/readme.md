# oak-athena

Oak language frontend for the **Athena** expression language (renamed from `oak-simple-math`).

SXO product tags stay `simple-math` / `sm` / optional `sxo` — same pattern as `oak-metis` ↔ CVO `.cvo`.

Surface:

- numbers, identifiers
- `+ - * / ^`, unary `-`, parentheses
- lowercase calls `sin(x)`, `cos(x)`
- lists `[a, b]`, dicts `{a: 1, "b": 2}`

Not Mathematica / Wolfram — see `oak-wolfram` for that dialect. Not the CAS crate `athena`.
