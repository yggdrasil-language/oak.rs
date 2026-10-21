#![doc = include_str!("readme.md")]
use crate::lsp::ScssLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for SCSS semantics (Stdio).
pub async fn serve_scss_mcp(vfs: MemoryVfs) {
    let service = ScssLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
