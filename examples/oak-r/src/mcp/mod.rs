#![doc = include_str!("readme.md")]
use crate::lsp::RLanguageService;
use oak_lsp::MemoryVfs;

pub async fn serve_r_mcp(vfs: MemoryVfs) {
    let service = RLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
