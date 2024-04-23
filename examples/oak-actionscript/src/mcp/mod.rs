#![doc = include_str!("readme.md")]

use crate::lsp::ActionScriptLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for ActionScript semantics (Stdio).
pub async fn serve_actionscript_mcp(vfs: MemoryVfs) {
    let service = ActionScriptLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
