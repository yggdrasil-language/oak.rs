#![doc = include_str!("readme.md")]
use crate::lsp::YamlLanguageService;
use oak_mcp::McpServer;
use oak_lsp::MemoryVfs;

/// Start an MCP server for YAML semantics (Stdio).
#[cfg(feature = "mcp")]
pub async fn serve_yaml_mcp(vfs: MemoryVfs) {
    let service = YamlLanguageService::new(vfs);
    let server = McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
