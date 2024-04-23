#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{NginxLanguage, parser::element_type::NginxElementType};
use oak_core::{Range, tree::RedNode};
#[cfg(feature = "lsp")]
use {
    oak_hover::HoverProvider,
    oak_lsp::{service::LanguageService, types::Hover},
    oak_lsp::Vfs,
    std::future::Future,
};
/// Hover provider implementation for Nginx.
#[cfg(feature = "lsp")]
pub struct NginxHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<NginxLanguage> for NginxHoverProvider {
    fn hover(&self, root: &RedNode<'_, NginxLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = root.green.kind;
        let contents = match kind {
            NginxElementType::Directive => "### Nginx Directive\nA configuration instruction for the Nginx server.",
            NginxElementType::Block => "### Nginx Block\nA group of directives enclosed in braces.",
            NginxElementType::Root => "### Nginx Configuration\nThe root of an Nginx configuration file.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(root.span()) })
    }
}
/// Language service implementation for Nginx.
#[cfg(feature = "lsp")]
pub struct NginxLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: NginxHoverProvider,
}
impl<V: Vfs> NginxLanguageService<V> {
    /// Creates a new Nginx language service with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: NginxHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for NginxLanguageService<V> {
    type Lang = NginxLanguage;
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
            let language = NginxLanguage::default();
            let parser = crate::parser::NginxParser::new(&language);
            let lexer = crate::lexer::NginxLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Option<Hover>> + Send + '_ {
        let uri = uri.to_string();
        async move {
            let h = self.with_root(&uri, |root| self.hover_provider.hover(&root, range)).await.flatten()?;
            Some(Hover { contents: h.contents, range: h.range })
        }
    }
}
