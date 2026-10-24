#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::JuliaLanguage;
#[cfg(feature = "lsp")]
use {futures::Future, oak_core::tree::RedNode, oak_lsp::Vfs, oak_lsp::service::LanguageService};
/// Language service implementation for Julia.
///
/// Provides IDE features such as diagnostics and workspace management
/// by integrating with the `oak-lsp` framework and `Vfs`.
#[cfg(feature = "lsp")]
pub struct JuliaLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
}
impl<V: Vfs> JuliaLanguageService<V> {
    /// Creates a new `JuliaLanguageService` with the given virtual file system.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for JuliaLanguageService<V> {
    type Lang = JuliaLanguage;
    type Vfs = V;
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
            let language = JuliaLanguage::default();
            let parser = crate::parser::JuliaParser::new(&language);
            let lexer = crate::lexer::JuliaLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
}
