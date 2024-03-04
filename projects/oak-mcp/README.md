# 馃殌 oak-mcp

[![Crates.io](https://img.shields.io/crates/v/oak-mcp.svg)](https://crates.io/crates/oak-mcp)
[![Documentation](https://docs.rs/oak-mcp/badge.svg)](https://docs.rs/oak-mcp)

**Model Context Protocol Integration for Oak** 鈥?Enable AI assistants to understand and analyze code through the MCP standard.

## 馃幆 Why oak-mcp?

The Model Context Protocol (MCP) is an open standard that enables AI assistants to interact with development tools and codebases. `oak-mcp` bridges Oak's parsing capabilities with MCP, allowing AI agents to perform code analysis and understanding tasks.

## 鉁?Key Features

- **馃 MCP Server Implementation** 鈥?Standard MCP server exposing Oak's language analysis
- **馃搳 Code Analysis Tools** 鈥?Parsing, symbol extraction, and navigation as MCP tools
- **馃攳 Semantic Understanding** 鈥?Query symbol definitions, references, and documentation
- **馃搧 Project-Wide Analysis** 鈥?Multi-file project understanding via `oak-vfs`
- **馃寪 Language Agnostic** 鈥?Works with any Oak language parser

## 馃彈锔?Architecture

### MCP Tools Provided

| Tool | Description |
|------|-------------|
| `parse_file` | Parse source file and return AST structure |
| `find_symbols` | Search for symbols matching a query |
| `get_definition` | Get definition location of a symbol |
| `find_references` | Find all references to a symbol |
| `get_hover` | Get hover information for a position |

## 馃敆 Ecosystem Integration

Integrates with `oak-core` for parsing, `oak-vfs` for file access, `oak-navigation` for code navigation, and any MCP-compatible AI assistant.

## 馃摉 Documentation

For usage examples and API details, see the [API documentation](https://docs.rs/oak-mcp).

## 馃 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
