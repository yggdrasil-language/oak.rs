#![doc = include_str!("readme.md")]
use crate::lsp::KotlinLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Kotlin semantics (Stdio).
pub async fn serve_kotlin_mcp(_vfs: MemoryVfs) {
    let service = KotlinLanguageService::new(crate::language::KotlinLanguage::default());
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
