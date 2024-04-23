#![doc = include_str!("readme.md")]
use crate::lsp::TailwindLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the MCP server (Stdio) for Tailwind semantics.
#[cfg(feature = "mcp")]
pub async fn serve_tailwind_mcp(vfs: MemoryVfs) {
    let service = TailwindLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
