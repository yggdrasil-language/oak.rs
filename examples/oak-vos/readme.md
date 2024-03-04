# `oak-vos`

`oak-vos` is the Oak CST-first frontend for VOS. It owns source structure and
syntax recovery only. VOS name resolution, type checking, contract identity,
UDF lowering, and YY execution lowering remain downstream responsibilities.

This crate deliberately does not route VOS through VOML, VOC, or VON, and it
does not define a database executor.

This initial structural CST slice is not a complete VOS grammar or Builder AST.
It must not be treated as a substitute for VOS conformance or used to admit
unvalidated source to an execution runtime. Existing VOS source parsers are
still legacy syntax implementations until replaced by CST-based builders.
