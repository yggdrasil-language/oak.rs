#![doc = include_str!("readme.md")]
use crate::lsp::LeanLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Lean semantics (Stdio).
pub async fn serve_lean_mcp(_vfs: MemoryVfs) {
    let service = LeanLanguageService::new(crate::language::LeanLanguage::default());
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
