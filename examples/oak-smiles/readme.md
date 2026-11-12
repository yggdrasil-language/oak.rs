# oak-smiles

SMILES and reaction-SMILES parser for the oak ecosystem.

The crate follows the same three-stage contract as the other oak frontends:

1. `SmilesLexer` emits atoms, bond symbols, branches, ring labels, dots, and
   arrows. Whitespace is skipped because it carries no meaning.
2. `SmilesParser` groups the token stream into a green tree of flat elements.
3. `SmilesBuilder` walks the elements in order and builds the molecular graph:
   the linear traversal, the branch stack, and the ring-label table together
   resolve every bond endpoint into a real atom index.

Traversal state is resolved in the builder, not guessed by the lexer: a ring
label remembers the atom it opened on and the order that was pending there, so
`C1CCCCC1` becomes six bonds with no text left over.

## Scope

Accepted: the organic subset (`B C N O P S F Cl Br I`), aromatic lowercase
atoms, bracket atoms with isotope, charge, explicit hydrogens and `@`/`@@`
chirality, single/double/triple/aromatic/any bonds, `/` and `\` bond stereo,
`%nn` ring labels, branches, `.`-separated fragments, `*` wildcards, and
reaction SMILES with `>` or `>>`.

Rejected with a positioned error: `%(...)` ring labels. The construct is kept
out of the suite rather than mis-parsed, and the message names it.

## Example

```rust
use oak_smiles::parse_smiles;

let root = parse_smiles("CCO").expect("parse");
assert_eq!(root.items[0].atoms.len(), 3);
assert_eq!(root.items[0].bonds.len(), 2);
```
