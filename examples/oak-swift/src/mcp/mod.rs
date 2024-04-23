#![doc = include_str!("readme.md")]
use crate::lsp::SwiftLanguageService;
use oak_lsp::MemoryVfs;

pub async fn serve_swift_mcp(vfs: MemoryVfs) {
    let service = SwiftLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
