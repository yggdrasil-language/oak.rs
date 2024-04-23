#![doc = include_str!("readme.md")]
use crate::lsp::HandlebarsLanguageService;
use oak_lsp::MemoryVfs;

/// Starts an MCP server for Handlebars semantics (Stdio).
#[cfg(feature = "io-std")]
pub async fn serve_handlebars_mcp(vfs: MemoryVfs) {
    let service = HandlebarsLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
