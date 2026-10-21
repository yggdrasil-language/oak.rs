#![doc = include_str!("readme.md")]
use oak_lsp::MemoryVfs;

#[cfg(feature = "io-std")]
use crate::lsp::OrgModeLanguageService;

/// Start an MCP server for Org-mode semantics (Stdio).
#[cfg(feature = "io-std")]
pub async fn serve_org_mode_mcp(vfs: MemoryVfs) {
    let service = OrgModeLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}

/// Start an MCP server for Org-mode semantics (Stdio).
#[cfg(not(feature = "io-std"))]
pub async fn serve_org_mode_mcp(_vfs: MemoryVfs) {
    panic!("MCP server requires io-std feature")
}
