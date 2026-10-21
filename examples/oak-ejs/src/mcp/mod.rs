#![doc = include_str!("readme.md")]
use crate::lsp::EjsLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for EJS semantics (Stdio).
pub async fn serve_ejs_mcp(vfs: MemoryVfs) {
    let service = EjsLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::stdout();

    server.run(reader, writer).await.unwrap()
}
