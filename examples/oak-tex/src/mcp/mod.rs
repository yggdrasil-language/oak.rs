#![doc = include_str!("readme.md")]
use crate::lsp::TexLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the TeX MCP server
pub async fn serve_tex_mcp(vfs: MemoryVfs) {
    let service = TexLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
