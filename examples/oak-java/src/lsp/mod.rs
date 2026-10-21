#![doc = include_str!("readme.md")]

#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::language::JavaLanguage;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {oak_lsp::LanguageService, oak_lsp::Vfs, std::future::Future};

/// Language service for Java.
#[cfg(feature = "lsp")]
pub struct JavaLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
}
impl<V: Vfs> JavaLanguageService<V> {
    /// Creates a new `JavaLanguageService`.
    pub fn new(vfs: V, _language: JavaLanguage) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for JavaLanguageService<V> {
    type Lang = JavaLanguage;
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
            let language = JavaLanguage::default();
            let parser = crate::parser::JavaParser::new(&language);
            let lexer = crate::lexer::JavaLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
}
