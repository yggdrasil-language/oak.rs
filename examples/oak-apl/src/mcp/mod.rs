#![doc = include_str!("readme.md")]
use crate::lsp::AplLanguageService;
use oak_lsp::MemoryVfs;

/// Serves the APL MCP (Model Context Protocol) server.
pub async fn serve_apl_mcp(vfs: MemoryVfs) {
    let service = AplLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
