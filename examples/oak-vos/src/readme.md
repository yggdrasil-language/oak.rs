# `oak-vos`

`oak-vos` is the Oak CST-first frontend for VOS. It owns source structure and
syntax recovery only. VOS name resolution, type checking, contract identity,
UDF lowering, and YY execution lowering remain downstream responsibilities.

This crate deliberately does not route VOS through VOML, VOC, or VON, and it
does not define a database executor.
