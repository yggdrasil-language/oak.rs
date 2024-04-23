#![doc = include_str!("readme.md")]
use crate::lsp::RbqLanguageService;
use oak_lsp::MemoryVfs;

/// Serves RBQ MCP service.
pub async fn serve_rbq_mcp(vfs: MemoryVfs) {
    let service = RbqLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
