#![doc = include_str!("readme.md")]
use crate::lsp::DejavuLanguageService;
use oak_lsp::MemoryVfs;

/// Starts Dejavu MCP service
pub async fn serve_dejavu_mcp(vfs: MemoryVfs) {
    let service = DejavuLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
