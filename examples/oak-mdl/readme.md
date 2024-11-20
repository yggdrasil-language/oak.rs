# oak-mdl

MDL connection-table parser for the oak ecosystem.

The crate follows the same three-stage contract as the other oak frontends:

1. `MdlLexer` classifies each physical line. It never interprets fixed columns,
   because column meaning depends on the line's position inside the record.
2. `MdlParser` groups the classified lines into a green tree: one `SourceFile`
   holding `HeaderLine`, `CountsLine`, `AtomLine`, `BondLine`, `PropertyLine`,
   `DataFieldName`, `DataValue`, and `RecordSeparator` elements.
3. `MdlBuilder` walks the green tree and produces `MdlRoot`: one `MdlRecord`
   per molfile or SDF member, with parsed atoms, bonds, properties, and data
   fields. Every node keeps its authored byte range.

Positions are taken from the counts line, exactly as the format requires: the
first three content lines are the header, the fourth is the counts line, then
`atom_count` atom lines and `bond_count` bond lines follow. Column meaning is
parsed only after the position is known, so a bond-looking counts line can
never be misclassified.

## Scope

Accepted: V2000 molfiles, SDF multi-record files separated by `$$$$`, `M  CHG`
/ `M  ISO` / `M  RAD` property lines, and `> <NAME>` data fields.

Rejected with a positioned error and no partial AST: V3000 records, `$RXN` and
`$RDFILE` reaction files. Reaction files are a separate node model; they are
refused rather than flattened into molecules, so a caller never reads a
reaction as if it were one structure.

## Example

```rust
use oak_mdl::parse_mdl;

let root = parse_mdl("water\n  test\n\n  1  0  0  0  0  0  0  0  0  0999 V2000\n").expect("parse");
assert_eq!(root.records.len(), 1);
```
