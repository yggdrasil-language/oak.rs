# Oak Glsl

GLSL support for the Oak language framework.

## Quick Start

```rust
use oak_glsl::parse_glsl;

let root = parse_glsl("void main() { }").expect("valid GLSL");
assert_eq!(root.items.len(), 1);
```
