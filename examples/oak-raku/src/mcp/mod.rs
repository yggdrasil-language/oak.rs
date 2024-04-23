#![doc = include_str!("readme.md")]
use crate::lsp::RakuLanguageService;
use oak_lsp::MemoryVfs;

/// Starts an MCP server (Stdio) for Raku semantics.
pub async fn serve_raku_mcp(vfs: MemoryVfs) {
    let service = RakuLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
