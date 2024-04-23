#![doc = include_str!("readme.md")]
use crate::HtmlLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
/// Highlighter module.
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::service::LanguageService,
    oak_lsp::Vfs,
};
/// MCP module.
/// Hover provider implementation for HTML.
///
/// This provider is responsible for generating hover information for HTML elements,
/// such as tag descriptions or attribute documentation.
#[cfg(feature = "lsp")]
pub struct HtmlHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<HtmlLanguage> for HtmlHoverProvider {
    /// Provides hover information for a specific node in the HTML tree.
    fn hover(&self, _node: &RedNode<HtmlLanguage>, _range: Range<usize>) -> Option<Hover> {
        None
    }
}
/// Language service implementation for HTML.
///
/// This service provides LSP features like hover, completion, and diagnostics
/// for the HTML language, utilizing the VFS for file management.
#[cfg(feature = "lsp")]
pub struct HtmlLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    _hover_provider: HtmlHoverProvider,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> HtmlLanguageService<V> {
    /// Creates a new `HtmlLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), _hover_provider: HtmlHoverProvider }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for HtmlLanguageService<V> {
    type Lang = HtmlLanguage;
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
            let language = HtmlLanguage::default();
            let parser = crate::parser::HtmlParser::new(&language);
            let lexer = crate::lexer::HtmlLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, _uri: &str, _range: Range<usize>) -> impl Future<Output = Option<oak_lsp::Hover>> + Send + '_ {
        async move { None }
    }
    fn completion(&self, _uri: &str, _offset: usize) -> impl Future<Output = Vec<oak_lsp::types::CompletionItem>> + Send + '_ {
        async move { vec![] }
    }
}
