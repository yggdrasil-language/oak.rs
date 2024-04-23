#![doc = include_str!("readme.md")]
use crate::lsp::SqlLanguageService;
use oak_lsp::MemoryVfs;

/// Starts an MCP (Model Context Protocol) server for SQL.
pub async fn serve_sql_mcp(vfs: MemoryVfs) {
    let service = SqlLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);
    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut writer = tokio::io::stdout();
    server.run(&mut reader, &mut writer).await.unwrap()
}
