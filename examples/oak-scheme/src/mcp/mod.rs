#![doc = include_str!("readme.md")]
use crate::lsp::SchemeLanguageService;
use oak_lsp::MemoryVfs;
use oak_mcp::McpServer;

/// Start an MCP server for Scheme semantics (Stdio).
pub async fn serve_scheme_mcp(vfs: MemoryVfs) {
    let service = SchemeLanguageService::new(vfs);
    let server = McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
