#![doc = include_str!("readme.md")]
use crate::lsp::RustLanguageService;
use oak_lsp::MemoryVfs;

/// Starts an Model Context Protocol (MCP) server for Rust semantics.
///
/// This server uses standard I/O (stdio) to communicate with clients and provides
/// semantic information about Rust source code managed by the virtual file system.
///
/// # Arguments
///
/// * `vfs` - The memory-based virtual file system used for document management.
pub async fn serve_rust_mcp(vfs: MemoryVfs) {
    let service = RustLanguageService::new(vfs);
    let server = oak_mcp::McpServer::new(service);

    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();
    let reader = tokio::io::BufReader::new(stdin);

    server.run(reader, stdout).await.unwrap()
}
