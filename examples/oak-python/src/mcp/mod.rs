#![doc = include_str!("readme.md")]
use crate::lsp::PythonLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Python semantics (Stdio).
pub async fn serve_python_mcp(vfs: MemoryVfs) {
    let service = PythonLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let mut reader = tokio::io::BufReader::new(tokio::io::stdin());
    let mut writer = tokio::io::stdout();

    server.run(&mut reader, &mut writer).await.unwrap()
}
