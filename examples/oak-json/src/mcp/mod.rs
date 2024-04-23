#![doc = include_str!("readme.md")]
use crate::lsp::JsonLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the MCP server for JSON semantics (Stdio).
#[cfg(feature = "mcp")]
pub async fn serve_json_mcp(vfs: MemoryVfs) {
    let service = JsonLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
