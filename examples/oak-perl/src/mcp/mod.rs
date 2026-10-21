#![doc = include_str!("readme.md")]
use crate::lsp::PerlLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Perl semantics (Stdio).
pub async fn serve_perl_mcp(vfs: MemoryVfs) {
    let service = PerlLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let reader = tokio::io::BufReader::new(stdin);

    server.run(reader, stdout).await.unwrap()
}
