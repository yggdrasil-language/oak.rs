# 📂 Oak VFS: Virtual File System

`oak-vfs` provides a unified Virtual File System (VFS) abstraction for the Oak language framework. It manages source files, metadata, and coordinate mapping (line/column to byte offset).

## 🚀 Features

- **Abstraction**: Unified interface for memory-based and disk-based storage.
- **Coordinate Mapping**: Efficiently convert between byte offsets and LSP-compatible line/column positions.
- **Metadata**: Access file types, lengths, and modification timestamps.
- **Change Tracking**: Watch for file system changes (disk-based VFS).
- **WASM Compatible**: Works in browser environments via `MemoryVfs`.

## 🚦 Quick Start

### Using Memory Vfs

```rust
use oak_core::source::Source;
use oak_vfs::{MemoryVfs, Vfs, WritableVfs};

let vfs = MemoryVfs::new();
vfs.write_file("file:///test.txt", "line 1\nline 2");

if let Some(source) = vfs.get_source("file:///test.txt") {
    println!("Content: {}", source.get_text_from(0));
}
```

### Coordinate Conversion

```rust
use oak_vfs::LineMap;
use oak_core::source::SourceText;

let source = SourceText::new("first line\nsecond line");
let line_map = LineMap::from_source(&source);

// Convert byte offset to (line, col)
let (line, col) = line_map.offset_to_line_col_utf16(&source, 15);
println!("Position: {}:{}", line, col);
```

## 🏗️ Core Components

- **[Vfs](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-vfs/src/lib.rs)**: The primary trait for file access.
- **[MemoryVfs](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-vfs/src/vfs/memory.rs)**: Thread-safe, in-memory implementation.
- **[DiskVfs](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-vfs/src/vfs/disk.rs)**: Physical file system access.
- **[LineMap](file:///e:/yydb%20%E6%95%B0%E6%8D%AE%E5%BA%93/oaks/projects/oak-vfs/src/line_map.rs)**: Fast lookup table for line starts.

## 🛠️ Architecture

The VFS layer sits between the raw source text and higher-level language services. It ensures that components like the Lexer and Parser can work with `Source` abstractions without worrying about where the data originates.
