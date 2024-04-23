#![doc = include_str!("readme.md")]
use crate::lsp::KokaLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Koka semantics (Stdio).
pub async fn serve_koka_mcp(_vfs: MemoryVfs) {
    let service = KokaLanguageService::new(crate::language::KokaLanguage::default());
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
