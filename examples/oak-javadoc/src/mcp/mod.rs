#![doc = include_str!("readme.md")]
#[cfg(feature = "lsp")]
use crate::lsp::JavadocLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the MCP server (Stdio) for Javadoc semantics.
#[cfg(feature = "mcp")]
pub async fn serve_javadoc_mcp(vfs: MemoryVfs) {
    #[cfg(feature = "lsp")]
    {
        let service = JavadocLanguageService::new(vfs);
        let server = oak_mcp::McpServer::new(service);

        let reader = tokio::io::BufReader::new(tokio::io::stdin());
        let writer = tokio::io::BufWriter::new(tokio::io::stdout());
        server.run(reader, writer).await.unwrap()
    }
}
