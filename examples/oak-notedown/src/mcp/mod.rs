#![doc = include_str!("readme.md")]
use crate::lsp::NoteLanguageService;
use oak_lsp::MemoryVfs;

/// Start an MCP server for Notedown semantics (Stdio).
pub async fn serve_note_mcp(vfs: MemoryVfs) {
    let service = NoteLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let reader = tokio::io::BufReader::new(tokio::io::stdin());
    let writer = tokio::io::BufWriter::new(tokio::io::stdout());
    server.run(reader, writer).await.unwrap()
}
