#![doc = include_str!("readme.md")]
use crate::lsp::WolframLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Wolfram semantics (Stdio).
#[cfg(feature = "mcp")]
pub async fn serve_wolfram_mcp(vfs: MemoryVfs) {
    let service = WolframLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
