#![doc = include_str!("readme.md")]
use crate::lsp::LLirLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for LLVM IR semantics (Stdio).
pub async fn serve_llir_mcp(vfs: MemoryVfs) {
    let service = LLirLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
