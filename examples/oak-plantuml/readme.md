# 🧩 Oak PlantUML Parser

PlantUML support for the Oak language framework, limited to the class and sequence diagram subsets used by diagram front ends.

## 🎯 Scope

`oak-plantuml` parses one `@startuml ... @enduml` block and produces a typed AST covering:

- **Class diagrams** — `class`, `interface`, `abstract class`, `enum`, optional `{ ... }` member blocks, and relations with multiplicities, labels and normalized arrow markers.
- **Sequence diagrams** — `participant` and `actor` declarations, four message arrows, and `activate` / `deactivate` spans.

The parser keeps every token in a lossless green tree, so whitespace and comments remain available to editors even though the typed AST is normalized.

## 🚫 Rejected Input

This crate is deliberately a strict subset. The upstream pre-scan is expected to reject:

- `skinparam`, `note`, `!include` and preprocessor directives
- activity, state, component, deployment and timing diagram constructs
- nested or multi-line member syntax beyond one member per line

Lines outside the `@startuml` / `@enduml` wrapper and lines the grammar cannot attribute are not silently dropped: the parser records a syntax diagnostic at the line start offset.

## 🚀 Quick Start

```rust,ignore
use oak_core::{Builder, Parser, SourceText, parser::session::ParseSession};
use oak_plantuml::{PlantUmlBuilder, PlantUmlLanguage, PlantUmlParser};

let source = SourceText::new(
    "@startuml\nclass Foo {\n+ int x\n}\nFoo --> Bar\n@enduml\n",
);
let language = PlantUmlLanguage::new();
let parser = PlantUmlParser::new(&language);
let mut cache = ParseSession::<PlantUmlLanguage>::default();
let parsed = parser.parse(&source, &[], &mut cache);
assert!(parsed.result.is_ok());

let builder = PlantUmlBuilder::new(&language);
let mut cache = ParseSession::<PlantUmlLanguage>::default();
let built = builder.build(&source, &[], &mut cache);
let root = built.result.expect("the document should build");
assert_eq!(root.diagrams.len(), 1);
```

## 🏗️ Architecture

The crate follows the Oak layering used by the other example languages:

1. `lexer` — line-aware tokens for keywords, arrows, strings, comments and punctuation.
2. `parser` — groups each statement into a composite green-tree element such as `ClassDeclaration`, `ClassMemberBlock`, `ClassRelation` or `SequenceMessage`.
3. `builder` — lowers the green tree into `PlantUmlRoot`, `ClassDiagram` / `SequenceDiagram` and span-carrying AST nodes.

## 🤝 Contributing

Issues and pull requests are welcome at [ygg-lang/oaks](https://github.com/ygg-lang/oaks).
