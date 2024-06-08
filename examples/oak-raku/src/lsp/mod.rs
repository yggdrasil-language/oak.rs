#![doc = include_str!("readme.md")]

pub mod formatter;
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{RakuLanguage, parser::element_type::RakuElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};

/// Raku hover provider implementation.
#[cfg(feature = "lsp")]
pub struct RakuHoverProvider;

#[cfg(feature = "lsp")]
impl HoverProvider<RakuLanguage> for RakuHoverProvider {
    fn hover(&self, node: &RedNode<RakuLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            RakuElementType::Statement => "### Raku Statement\nA single execution unit in Raku.",
            RakuElementType::Expression => "### Raku Expression\nA piece of code that evaluates to a value.",
            _ => return None,
        };

        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}

/// Raku language service implementation.
#[cfg(feature = "lsp")]
pub struct RakuLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: RakuHoverProvider,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> RakuLanguageService<V> {
    /// Creates a new `RakuLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: RakuHoverProvider }
    }
}

#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for RakuLanguageService<V> {
    type Lang = RakuLanguage;
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

    fn hover_provider(&self) -> Option<&(dyn HoverProvider<RakuLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
