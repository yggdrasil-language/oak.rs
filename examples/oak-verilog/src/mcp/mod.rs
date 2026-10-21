#![doc = include_str!("readme.md")]
use crate::VerilogLanguage;
use oak_lsp::service::LanguageService;
use oak_mcp::McpServer;
use oak_lsp::Vfs;

/// Serves Verilog MCP.
pub async fn serve_verilog_mcp<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession>(vfs: V) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    crate::lsp::VerilogLanguageService<V>: LanguageService,
{
    let service = crate::lsp::VerilogLanguageService::new(vfs);
    let _server = McpServer::new(service);
    Ok(())
}
