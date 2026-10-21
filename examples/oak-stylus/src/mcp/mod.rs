#![doc = include_str!("readme.md")]
use crate::lsp::StylusLanguageService;
use oak_mcp::McpServer;
use oak_lsp::Vfs;

pub async fn serve_stylus_mcp<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession>(vfs: V) {
    let service = StylusLanguageService::new(vfs);
    let server = McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
