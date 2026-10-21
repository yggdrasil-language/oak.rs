#![doc = include_str!("readme.md")]
use crate::lsp::JLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the MCP server for the J language.
pub async fn serve_j_mcp(vfs: MemoryVfs) {
    let service = JLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
