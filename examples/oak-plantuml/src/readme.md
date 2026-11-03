# 🛠️ PlantUML Parser Developer Guide

`oak-plantuml` implements the Oak lexer / parser / builder pipeline for a strict PlantUML subset.

## 📐 Grammar Surface

The parser accepts exactly one `@startuml ... @enduml` wrapper. An optional name after `@startuml` is consumed and ignored.

### Class diagrams

```text
class Foo
class "Display Name" as Alias {
+ int x
- void run()
# internal state
}
interface Service
abstract class Base
enum Color
Foo "1" --> "many" Bar : uses
```

- `+`, `-` and `#` select public, private and protected visibility.
- Member text is everything after the visibility marker, trimmed. A member is a method when its text contains `(`.
- Relations normalize `-->`, `<--`, `<|--`, `--|>`, `*--`, `--*`, `o--`, `--o`, `..>`, `<..` and `..` into `ClassRelationKind` plus `marker_at_left`.

### Sequence diagrams

```text
participant "Web UI" as UI
actor DB
UI ->> DB : query
activate DB
DB --> UI : rows
deactivate DB
```

- `->`, `->>`, `-->` and `-->>` map to `Solid`, `SolidArrow`, `Dotted` and `DottedArrow`.
- `activate` / `deactivate` produce closed `ActivationSpan` indices over the message list.
- Lines starting with `'` are comments.

A document is treated as a sequence diagram when its first effective statement is a participant, actor, activation or message line. Otherwise it is treated as a class diagram.

## 🧱 Tree Shape

Each statement is one composite element:

| Element | Contents |
|---|---|
| `StartMarker` / `EndMarker` | Wrapper token and optional diagram name |
| `ClassDeclaration` | Keyword, name or alias, optional `ClassMemberBlock` |
| `ClassMemberBlock` | Braces and one `ClassMember` per line |
| `ClassMember` | Visibility token and member text tokens |
| `ClassRelation` | Endpoints, optional multiplicities, arrow, optional label |
| `ParticipantDeclaration` | `participant` / `actor`, name and optional alias |
| `SequenceMessage` | Endpoints, arrow and optional text |
| `Activate` / `Deactivate` | Keyword and participant name |
| `Comment` | One PlantUML line comment |

Unrecognized lines become `Error` elements and add a syntax diagnostic at the line start offset.

## 🔍 Typed AST

The `ast` module exposes `PlantUmlRoot { diagrams }`, where each diagram is either `ClassDiagram` or `SequenceDiagram`. Every declaration, relation, message and participant carries a byte `Range<usize>`.

## ✅ Testing

```text
cargo test -p oak-plantuml
```

The `tests` directory covers tokenization, grouped parser elements and typed AST construction for both diagram kinds.
