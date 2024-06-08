#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::{DocumentSession, Vfs},
    oak_lsp::{service::LanguageService, workspace::WorkspaceManager},
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::RhombusLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for Rhombus.
#[cfg(feature = "lsp")]
pub struct RhombusHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<RhombusLanguage> for RhombusHoverProvider {
    fn hover(&self, node: &RedNode<RhombusLanguage>, _range: Range<usize>) -> Option<Hover> {
        let _kind = node.green.kind;
        // Basic Rhombus hover implementation
        None
    }
}
/// Language service implementation for Rhombus.
#[cfg(feature = "lsp")]
pub struct RhombusLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: RhombusHoverProvider,
}
impl<V: Vfs> RhombusLanguageService<V> {
    /// Creates a new `RhombusLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: RhombusHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for RhombusLanguageService<V> {
    type Lang = RhombusLanguage;
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
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<RhombusLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
