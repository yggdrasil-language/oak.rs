# oak-abc

ABC music notation parser for the oak ecosystem.

The crate follows the same three-stage contract as the other oak frontends:

1. `AbcLexer` emits one token per physical line. ABC is line-oriented: header
   fields, music, lyric lines, and directives each own their whole line, so a
   finer token split would only have to be joined back together.
2. `AbcParser` groups the lines into a green tree of `Line` elements.
3. `AbcBuilder` dispatches on the line prefix and parses the line body with byte
   offsets, so every parsed note, bar line, and refused construct keeps the
   range of the text that produced it.

Tune structure, voices, and measures are resolved in the builder: `X:` starts a
tune, `V:` starts a voice within it, and bar tokens close measures.

## Scope

Accepted: `X`, `T`, `C`, `Q`, `M`, `L`, `K`, and `V` header fields; notes with
octave marks (`'` and `,`) and accidentals (`^`, `^^`, `_`, `__`, `=`); rests
(`z` and `x`); chords; durations including fractional forms and dots; ties;
slurs; tuplets (`(3`); bar lines (`|`, `||`, `|]`, `|:`, `:|`, `::`); the
staccato marker and `!`/`+` decoration groups; and `w:` lyric lines aligned to
the preceding music line.

Refused as explicit `Unsupported` nodes, never dropped: `%%` directives, grace
notes, chord symbols in double quotes, inline field changes, and `W:` text
blocks. A refusal names the construct and keeps the authored text.

## Example

```rust
use oak_abc::parse_abc;

let root = parse_abc("X:1\nT:Smoke\nM:4/4\nL:1/4\nK:C\nCDEF|").expect("parse");
assert_eq!(root.tunes.len(), 1);
assert_eq!(root.tunes[0].voices[0].measures.len(), 1);
```
