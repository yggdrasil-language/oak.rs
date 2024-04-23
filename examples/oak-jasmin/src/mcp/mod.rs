#![doc = include_str!("readme.md")]
use crate::lsp::JasminLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the Jasmin MCP server.
pub async fn serve_jasmin_mcp(vfs: MemoryVfs) {
    let service = JasminLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
