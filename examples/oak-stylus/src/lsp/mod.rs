#![doc = include_str!("readme.md")]
use crate::language::StylusLanguage;
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
/// Formatter module.
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
#[cfg(feature = "lsp")]
pub struct StylusHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<StylusLanguage> for StylusHoverProvider {
    fn hover(&self, _node: &RedNode<StylusLanguage>, _range: Range<usize>) -> Option<Hover> {
        None
    }
}
#[cfg(feature = "lsp")]
pub struct StylusLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: StylusHoverProvider,
}
impl<V: Vfs> StylusLanguageService<V> {
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: StylusHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for StylusLanguageService<V> {
    type Lang = StylusLanguage;
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
            let language = StylusLanguage::default();
            let parser = crate::parser::StylusParser::new(&language);
            let lexer = crate::lexer::StylusLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<StylusLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
