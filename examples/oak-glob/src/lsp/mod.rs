#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "oak-pretty-print")]
pub mod formatter;

#[cfg(feature = "oak-highlight")]
pub use highlighter::GlobHighlighter;

#[cfg(feature = "oak-pretty-print")]
pub use formatter::GlobFormatter;

#[cfg(feature = "lsp")]
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use oak_lsp::service::LanguageService;
#[cfg(feature = "lsp")]
use oak_lsp::{Vfs, DocumentSession};
#[cfg(feature = "lsp")]
use std::future::Future;

#[cfg(feature = "lsp")]
/// Language service for glob patterns.
pub struct GlobLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> GlobLanguageService<V> {
    /// Creates a new `GlobLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}

#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + DocumentSession> LanguageService for GlobLanguageService<V> {
    type Lang = crate::language::GlobLanguage;
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
