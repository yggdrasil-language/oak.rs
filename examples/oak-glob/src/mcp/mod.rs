use oak_lsp::service::LanguageService;
use oak_mcp::McpServer;
use oak_vfs::Vfs;

/// Serves the glob pattern MCP service.
pub async fn serve_glob_mcp<V: Vfs + Send + Sync + 'static + oak_vfs::DocumentSession>(vfs: V) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    crate::lsp::GlobLanguageService<V>: LanguageService,
{
    let service = crate::lsp::GlobLanguageService::new(vfs);
    let server = McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.map_err(|e| e.into())
}
