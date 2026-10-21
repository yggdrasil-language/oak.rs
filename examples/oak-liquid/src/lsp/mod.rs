/// LSP module for Liquid
///
/// This module provides LSP (Language Server Protocol) support for Liquid templates.

#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {oak_core::tree::RedNode, oak_lsp::LanguageService, oak_lsp::MemoryVfs};

#[cfg(feature = "oak-pretty-print")]
pub mod formatter;

use crate::language::LiquidLanguage;

/// Liquid language service.
#[cfg(feature = "lsp")]
pub struct LiquidLanguageService {
    vfs: MemoryVfs,
    workspace: oak_lsp::workspace::WorkspaceManager,
}

#[cfg(feature = "lsp")]
impl LiquidLanguageService {
    /// Creates a new language service.
    pub fn new(vfs: MemoryVfs) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}

#[cfg(feature = "lsp")]
impl LanguageService for LiquidLanguageService {
    type Lang = LiquidLanguage;
    type Vfs = MemoryVfs;

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
