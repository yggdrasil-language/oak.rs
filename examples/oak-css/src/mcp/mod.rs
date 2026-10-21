#![doc = include_str!("readme.md")]
use oak_lsp::service::LanguageService;
use oak_mcp::McpServer;
use oak_lsp::Vfs;

/// Serves CSS MCP.
pub async fn serve_css_mcp<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession>(vfs: V) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    crate::lsp::CssLanguageService<V>: LanguageService,
{
    let service = crate::lsp::CssLanguageService::new(vfs);
    let server = McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap();
    Ok(())
}
