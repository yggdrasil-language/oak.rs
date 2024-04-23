#![doc = include_str!("readme.md")]
use crate::lsp::PhpLanguageService;
use oak_lsp::MemoryVfs;

/// PHP MCP server implementation.
///
/// This module provides integration with the Model Context Protocol (MCP)
/// for PHP language services.
pub async fn serve_php_mcp(vfs: MemoryVfs) {
    let service = PhpLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
