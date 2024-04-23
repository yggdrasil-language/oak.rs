#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {oak_lsp::LanguageService, oak_lsp::MemoryVfs};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::language::ValaLanguage;
/// Vala language service.
#[cfg(feature = "lsp")]
pub struct ValaLanguageService {
    vfs: MemoryVfs,
    workspace: oak_lsp::workspace::WorkspaceManager,
}
#[cfg(feature = "lsp")]
impl ValaLanguageService {
    /// Creates a new Vala language service.
    pub fn new(vfs: MemoryVfs) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}
#[cfg(feature = "lsp")]
impl LanguageService for ValaLanguageService {
    type Lang = ValaLanguage;
    type Vfs = MemoryVfs;
    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }
    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }
    fn with_root<R, F>(&self, _uri: &str, _f: F) -> impl std::future::Future<Output = Option<R>> + Send
    where
        R: Send,
        F: FnOnce(oak_core::tree::RedNode<'_, Self::Lang>) -> R + Send,
    {
        async { None }
    }
}
