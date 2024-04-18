#![doc = include_str!("readme.md")]
use crate::LessLanguage;
use oak_core::tree::RedNode;
/// Highlighter module.
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {oak_lsp::service::LanguageService, oak_vfs::Vfs, std::future::Future};
/// MCP module.
/// Formatter module.
#[cfg(feature = "lsp")]
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
/// Language service implementation for Less.
#[cfg(feature = "lsp")]
pub struct LessLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> LessLanguageService<V> {
    /// Creates a new `LessLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_vfs::DocumentSession> LanguageService for LessLanguageService<V> {
    type Lang = LessLanguage;
    type Vfs = V;
    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }
    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }
    fn with_root<R, F>(&self, _uri: &str, _f: F) -> impl Future<Output = Option<R>> + Send
    where
        R: Send,
        F: FnOnce(RedNode<'_, Self::Lang>) -> R + Send,
    {
        async { None }
    }
}
