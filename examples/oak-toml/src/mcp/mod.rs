#![doc = include_str!("readme.md")]
use crate::lsp::TomlLanguageService;
use oak_lsp::MemoryVfs;

/// Starts the MCP server (Stdio) for TOML semantics.
#[cfg(feature = "mcp")]
pub async fn serve_toml_mcp(vfs: MemoryVfs) {
    let service = TomlLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut writer = tokio::io::stdout();

    server.run(&mut reader, &mut writer).await.unwrap()
}
