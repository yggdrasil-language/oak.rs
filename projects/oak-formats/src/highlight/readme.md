# highlight

Frontend-neutral syntax highlighting contracts (aligned with JetBrains IDE platform + C# `Nyar.Analyzer.Highlight`).

## Layers

| Layer           | Location                           | Role                                                                                              |
|:----------------|:-----------------------------------|:--------------------------------------------------------------------------------------------------|
| Platform        | `nyar-analyzer::highlight`         | `HighlightKind`, `HighlightSpan`, `Highlighter` trait, `HighlighterKind`, `Registry`, `hl-*` HTML |
| Language plugin | any frontend crate | Multiple passes per language: `Lexical` + `Semantic`                                          |

## Multi-pass highlighting

One `language_id` may register several highlighters:

- **Lexical** — source text only, synchronous and cheap (SSG docs, typing)
- **Semantic** — uses `AnalysisContext` to upgrade identifiers to types, functions, etc. (IDE / LSP)

`HighlighterRegistry::highlight_merged` applies lexical first, then semantic overlay.

Concrete languages **must not** be implemented inside `nyar-analyzer`; the analyzer also does not depend on `std-data`
lexers directly.
