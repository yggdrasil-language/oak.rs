#![doc = include_str!("readme.md")]
use crate::lsp::JavaScriptLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for JavaScript semantics (Stdio).
pub async fn serve_javascript_mcp(vfs: MemoryVfs) {
    let service = JavaScriptLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::stdout();

    server.run(reader, writer).await.unwrap()
}
