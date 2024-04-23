#![doc = include_str!("readme.md")]
use crate::lsp::PowerShellLanguageService;
use oak_lsp::MemoryVfs;

/// Serves the PowerShell MCP (Model Context Protocol) server.
pub async fn serve_powershell_mcp(vfs: MemoryVfs) {
    let service = PowerShellLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
