# Valkyrie Lexer Module

The Valkyrie Lexer module provides advanced lexical analysis for the Valkyrie language. It converts raw source text into a stream of tokens, serving as the foundation for the Valkyrie compiler and toolchain.

## Purpose

The primary goal of this module is to provide a fast, accurate, and feature-rich tokenizer for Valkyrie. It handles modern language features, including sophisticated numeric literals, string interpolation, and complex identifier rules, while maintaining high performance.

## Features

- **Keyword Recognition**: Supports all Valkyrie keywords for declarations, control flow, and type systems.
- **Advanced Literals**: Parses decimal, hexadecimal, binary, and octal numbers, including underscores for readability.
- **String Support**: Handles single-quoted, double-quoted, and multi-line strings with escape sequences.
- **Interpolation Recognition**: Correctly identifies interpolation markers within string literals.
- **Comment Processing**: Supports single-line (`//`) and multi-line (`/* ... */`) comments.
- **Whitespace Sensitivity**: Correctly handles whitespace-significant constructs where applicable.
- **Precise Source Mapping**: Each token contains detailed span information for accurate error reporting and IDE features.

## Token Types

### Keywords
- **Declarations**: `let`, `micro`, `data`, `type`, `trait`, `impl`, `module`, `import`.
- **Visibility & Modifiers**: `pub`, `mut`, `async`, `await`, `static`.
- **Control Flow**: `if`, `else`, `for`, `in`, `while`, `loop`, `match`, `return`, `break`, `continue`.
- **Type System**: `Int`, `Float`, `String`, `Bool`, `Char`, `Unit`.

### Literals
- **Numeric**: `42`, `3.14159`, `0xFF_AA_BB`, `0b1010_1100`.
- **String**: `"Hello Valkyrie"`, `'c'`, `"""Multi-line string"""`.
- **Boolean**: `true`, `false`.

### Operators & Symbols
- **Arithmetic**: `+`, `-`, `*`, `/`, `%`.
- **Comparison**: `==`, `!=`, `<`, `>`, `<=`, `>=`, `<=>`.
- **Logical**: `&&`, `||`, `!`.
- **Assignment**: `=`, `+=`, `-=`, `*=`, `/=`, `:=`.
- **Navigation**: `.`, `::`, `->`, `=>`.
- **Structural**: `(`, `)`, `[`, `]`, `{`, `}`, `,`, `:`, `;`.

## Usage Example

```rust
use oak_valkyrie::lexer::ValkyrieLexer;

fn main() {
    let valkyrie_source = r#"
        module Main
        
        pub micro greet(name: String) {
            println("Hello, ${name}!")
        }
    "#;

    let mut lexer = ValkyrieLexer::new();
    let tokens = lexer.tokenize(valkyrie_source);

    for token in tokens {
        println!("{:?}: '{}' at {:?}", token.token_type, token.lexeme, token.span);
    }
}
```

## Error Handling

The lexer detects and reports:
- **Invalid Characters**: Characters not allowed in Valkyrie source files.
- **Unterminated Literals**: Unclosed strings, characters, or comments.
- **Malformed Numbers**: Incorrectly formatted numeric literals.
- **Position Context**: All errors are accompanied by precise source coordinates.

## Design Principles

1. **Modernity**: Designed to support modern language features and syntax.
2. **Speed**: Optimized for fast tokenization of large projects.
3. **Accuracy**: Strictly follows the Valkyrie language specification.
4. **Tool-Friendly**: Provides rich metadata for IDEs and static analysis tools.
