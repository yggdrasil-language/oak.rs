#![doc = include_str!("readme.md")]
use crate::lsp::TwigLanguageService;
use oak_mcp::McpServer;
use oak_lsp::MemoryVfs;

/// Starts the Twig MCP service
pub async fn serve_twig_mcp(vfs: MemoryVfs) {
    let service = TwigLanguageService::new(vfs);
    let server = McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
