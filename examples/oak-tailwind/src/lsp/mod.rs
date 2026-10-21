#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::language::TailwindLanguage;

#[cfg(feature = "lsp")]
use {
    oak_core::tree::RedNode,
    oak_lsp::LanguageService,
    oak_vfs::{MemoryVfs, Vfs},
    std::future::Future,
};

#[cfg(feature = "lsp")]
/// Language service implementation for Tailwind CSS.
#[cfg(feature = "lsp")]
pub struct TailwindLanguageService {
    vfs: MemoryVfs,
    workspace: oak_lsp::workspace::WorkspaceManager,
}

#[cfg(feature = "lsp")]
impl TailwindLanguageService {
    /// Creates a new `TailwindLanguageService` with the given VFS.
    pub fn new(vfs: MemoryVfs) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}

#[cfg(feature = "lsp")]
impl LanguageService for TailwindLanguageService {
    type Lang = TailwindLanguage;
    type Vfs = MemoryVfs;

    fn vfs(&self) -> &Self::Vfs {
        &self.vfs
    }

    fn workspace(&self) -> &oak_lsp::workspace::WorkspaceManager {
        &self.workspace
    }

    fn with_root<R, F>(&self, uri: &str, f: F) -> impl Future<Output = Option<R>> + Send
    where
        R: Send,
        F: FnOnce(RedNode<'_, Self::Lang>) -> R + Send,
    {
        let source = self.vfs().request_source(uri);
        async move {
            let source = source?;
            let language = TailwindLanguage::default();
            let parser = crate::parser::TailwindParser::new(language);
            let lexer = crate::lexer::TailwindLexer::new(language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }

    fn completion(&self, _uri: &str, _offset: usize) -> impl std::future::Future<Output = Vec<oak_lsp::types::CompletionItem>> + Send + '_ {
        async move { vec![] }
    }
}
