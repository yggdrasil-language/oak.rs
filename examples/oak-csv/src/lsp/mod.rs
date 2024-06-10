#![doc = include_str!("readme.md")]
use crate::language::CsvLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {futures::Future, oak_lsp::Vfs, oak_lsp::service::LanguageService, oak_lsp::types::Hover as LspHover};
#[cfg(feature = "lsp")]
/// Language service implementation for CSV.
pub struct CsvLanguageService<V: Vfs> {
    /// The virtual file system.
    vfs: V,
    /// The workspace manager.
    workspace: oak_lsp::workspace::WorkspaceManager,
}
impl<V: Vfs> CsvLanguageService<V> {
    /// Creates a new `CsvLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::new() }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for CsvLanguageService<V> {
    type Lang = CsvLanguage;
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
            let parser = crate::parser::CsvParser::new();
            let lexer = crate::lexer::CsvLexer::new();
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, _uri: &str, _range: Range<usize>) -> impl Future<Output = Option<LspHover>> + Send + '_ {
        async move { None }
    }
}
