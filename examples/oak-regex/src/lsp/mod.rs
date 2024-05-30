#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::language::RegexLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for regular expressions.
#[cfg(feature = "lsp")]
pub struct RegexHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<RegexLanguage> for RegexHoverProvider {
    /// Provides hover information for a regex node.
    fn hover(&self, _node: &RedNode<'_, RegexLanguage>, _range: Range<usize>) -> Option<Hover> {
        None
    }
}
/// Language service implementation for regular expressions.
#[cfg(feature = "lsp")]
pub struct RegexLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: RegexHoverProvider,
}
impl<V: Vfs> RegexLanguageService<V> {
    /// Creates a new `RegexLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: RegexHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for RegexLanguageService<V> {
    type Lang = RegexLanguage;
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
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<RegexLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
    fn completion(&self, _uri: &str, _offset: usize) -> impl Future<Output = Vec<oak_lsp::types::CompletionItem>> + Send + '_ {
        async move { vec![] }
    }
}
