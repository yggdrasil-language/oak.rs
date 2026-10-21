#![doc = include_str!("readme.md")]
use crate::lsp::NixLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Nix semantics (Stdio).
pub async fn serve_nix_mcp(vfs: MemoryVfs) {
    let service = NixLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
