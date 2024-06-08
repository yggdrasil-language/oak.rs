#![doc = include_str!("readme.md")]
use crate::lsp::RhombusLanguageService;
use oak_lsp::MemoryVfs;
use oak_mcp::McpServer;

/// Start an MCP server for Rhombus semantics (Stdio).
pub async fn serve_rhombus_mcp(vfs: MemoryVfs) {
    let service = RhombusLanguageService::new(vfs);
    let server = McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
