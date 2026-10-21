#![doc = include_str!("readme.md")]
use crate::lsp::ValaLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the Vala MCP service.
pub async fn serve_vala_mcp(vfs: MemoryVfs) {
    let service = ValaLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
