#![doc = include_str!("readme.md")]
use crate::lsp::MatlabLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for MATLAB semantics (Stdio).
pub async fn serve_matlab_mcp(vfs: MemoryVfs) {
    let service = MatlabLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
