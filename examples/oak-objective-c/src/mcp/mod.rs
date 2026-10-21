#![doc = include_str!("readme.md")]
use crate::lsp::ObjectiveCLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Objective-C semantics (Stdio).
pub async fn serve_objective_c_mcp(vfs: MemoryVfs) {
    let service = ObjectiveCLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
