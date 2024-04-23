#![doc = include_str!("readme.md")]
use crate::{Dsv, language::DsvLanguage};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {futures::Future, oak_lsp::service::LanguageService, oak_lsp::types::Hover as LspHover, oak_lsp::Vfs};
#[cfg(feature = "lsp")]
/// Language service implementation for DSV.
pub struct DsvLanguageService<const LANG: DsvLanguage, V: Vfs> {
    /// The virtual file system.
    vfs: V,
    /// The workspace manager.
    workspace: oak_lsp::workspace::WorkspaceManager,
}
impl<const LANG: DsvLanguage, V: Vfs> DsvLanguageService<LANG, V> {
    /// Creates a new `DsvLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::new() }
    }
}
impl<const LANG: DsvLanguage, V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for DsvLanguageService<LANG, V> {
    type Lang = Dsv<LANG>;
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
    fn hover(&self, _uri: &str, _range: Range<usize>) -> impl Future<Output = Option<LspHover>> + Send + '_ {
        async move { None }
    }
}
