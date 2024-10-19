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

- `+`, `-`, `#` and `~` select public, private, protected and package visibility.
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

### Activity diagrams

Both the current and the legacy syntax are accepted.

```text
start
:Collect input;
if (valid?) then (yes)
:Store;
else (no)
:Reject;
endif
fork
:Path A;
fork again
:Path B;
end fork
|Customer|
:Order;
|Warehouse|
:Ship;
stop
```

```text
(*) --> "First"
"First" --> "Second" : next
"Second" --> (*)
```

- `start` / `stop` / `end` become `ActivityNode::Start` and `ActivityNode::Stop`.
- `:text;` becomes `ActivityAction` with the colon and the trailing semicolon removed.
- `if` / `elseif` / `else` / `endif` become a nested `ActivityDecision`, one branch per clause, in source order.
- `fork` / `fork again` / `end fork` (also `end merge` and `end split`) become a nested `ActivityFork` with one branch per parallel path.
- `|Name|` and `|#color|Name|` start an `ActivitySwimlane` that owns the following nodes until the next lane or block end.
- Legacy transitions split on `-->`, treat a leading `(*)` as the start pseudo state and a trailing `(*)` as the stop pseudo state.

### State diagrams

```text
[*] --> Idle
state Idle
state Processing {
  [*] --> Reading
  Reading --> Writing
  --
  [*] --> Retrying
}
Processing --> Idle : done
state Forked <<fork>>
```

- `state` declarations carry an optional quoted display name, an optional `as` alias and a `<<fork>>`, `<<join>>`, `<<choice>>`, `<<start>>`, `<<end>>`, `<<entryPoint>>` or `<<exitPoint>>` stereotype.
- A composite state keeps its braces and every `--` starts a new entry in `regions`, so concurrent regions are explicit.
- `[*]` becomes `StateEndpoint::Start` on the left of an arrow and `StateEndpoint::End` on the right.

### Component diagrams

```text
package "Backend" as B {
  component "API Gateway" as API
  component Service
  port http
  portin input
  portout metrics
  interface Repository
}
API --> Repository : calls
```

- `component`, `[Name]` and `interface` declarations accept an optional quoted display name and an `as` alias.
- `port`, `portin` and `portout` map to `PortDirection::InOut`, `In` and `Out`.
- A `package ... { }` block nests its declarations in `ComponentPackage::elements`.
- Relations reuse `ClassRelationKind` and carry optional multiplicities and a label.

### Family detection

A document is treated as a sequence diagram when its first effective statement is a participant, actor, activation or message line. Component, state and activity markers at the start of a line take precedence over that default. Otherwise the document is treated as a class diagram. Mixing class and sequence declarations in one source is rejected.

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
| `ActivityDiagram` / `StateDiagram` / `ComponentDiagram` | Container spanning every statement of one diagram |
| `ActivityStatement` / `StateStatement` / `ComponentStatement` | One statement line, including nested block braces |
| `Comment` | One PlantUML line comment |

Line-oriented families keep every statement as a flat line element. Nesting is rebuilt by the builder from braces, `--` region separators and `if` / `fork` terminators.
Unrecognized lines inside a class or sequence diagram become `Error` elements and add a syntax diagnostic at the line start offset.

## 🔍 Typed AST

The `ast` module exposes `PlantUmlRoot { diagrams }`, where each diagram is a `ClassDiagram`, `SequenceDiagram`, `ActivityDiagram`, `StateDiagram` or `ComponentDiagram`. Every declaration, relation, message, activity node and state carries a byte `Range<usize>`.

## ✅ Testing

```text
cargo test -p oak-plantuml
```

The `tests` directory covers tokenization, grouped parser elements and typed AST construction for every diagram family.
