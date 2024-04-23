#![doc = include_str!("readme.md")]
use crate::lsp::ScalaLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Scala semantics (Stdio).
pub async fn serve_scala_mcp(vfs: MemoryVfs) {
    let service = ScalaLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
