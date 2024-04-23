#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_core::tree::RedNode,
    oak_lsp::LanguageService,
    oak_lsp::{MemoryVfs, Vfs},
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::language::TwigLanguage;
/// Twig language service.
#[cfg(feature = "lsp")]
pub struct TwigLanguageService {
    vfs: MemoryVfs,
    workspace: oak_lsp::workspace::WorkspaceManager,
}
#[cfg(feature = "lsp")]
impl TwigLanguageService {
    /// Creates a new language service.
    pub fn new(vfs: MemoryVfs) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}
#[cfg(feature = "lsp")]
impl LanguageService for TwigLanguageService {
    type Lang = TwigLanguage;
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
            let language = TwigLanguage::default();
            let parser = crate::parser::TwigParser::new(&language);
            let lexer = crate::lexer::TwigLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
}
