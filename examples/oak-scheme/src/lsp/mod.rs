#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::{service::LanguageService, workspace::WorkspaceManager},
    oak_vfs::{Vfs, WritableVfs},
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::SchemeLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for Scheme.
#[cfg(feature = "lsp")]
pub struct SchemeHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<SchemeLanguage> for SchemeHoverProvider {
    fn hover(&self, node: &RedNode<SchemeLanguage>, _range: Range<usize>) -> Option<Hover> {
        let _kind = node.green.kind;
        // Basic Scheme hover implementation
        None
    }
}
/// Language service implementation for Scheme.
#[cfg(feature = "lsp")]
pub struct SchemeLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: SchemeHoverProvider,
}
impl<V: Vfs> SchemeLanguageService<V> {
    /// Creates a new `SchemeLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: SchemeHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for SchemeLanguageService<V> {
    type Lang = SchemeLanguage;
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
        let source = self.vfs().get_source(uri);
        async move {
            let source = source?;
            let language = SchemeLanguage::default();
            let parser = crate::parser::SchemeParser::new(&language);
            let lexer = crate::lexer::SchemeLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<SchemeLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
