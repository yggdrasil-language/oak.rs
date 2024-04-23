#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::service::LanguageService,
    oak_lsp::Vfs,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::language::SolidityLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
pub struct SolidityHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<SolidityLanguage> for SolidityHoverProvider {
    fn hover(&self, node: &RedNode<SolidityLanguage>, _range: Range<usize>) -> Option<Hover> {
        let _kind = node.green.kind;
        // Solidity specific kinds would go here
        None
    }
}
#[cfg(feature = "lsp")]
pub struct SolidityLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: SolidityHoverProvider,
}
impl<V: Vfs> SolidityLanguageService<V> {
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: SolidityHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for SolidityLanguageService<V> {
    type Lang = SolidityLanguage;
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
        // Solidity lexer/parser traits are not wired yet.
        async { None }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<SolidityLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
