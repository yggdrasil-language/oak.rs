#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover as ProviderHover, HoverProvider},
    oak_lsp::{service::LanguageService, types::Hover as LspHover},
    oak_vfs::Vfs,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::{SassLanguage, parser::element_type::SassElementType};
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for Sass.
#[cfg(feature = "lsp")]
pub struct SassHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<SassLanguage> for SassHoverProvider {
    fn hover(&self, node: &RedNode<SassLanguage>, _range: Range<usize>) -> Option<ProviderHover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            SassElementType::Selector => "### Sass Selector\nSelects HTML elements to style.",
            SassElementType::Variable => "### Sass Variable\nStores a reusable CSS value.",
            SassElementType::Mixin => "### Sass Mixin\nDefines a reusable group of CSS declarations.",
            SassElementType::Function => "### Sass Function\nReturns a value based on arguments.",
            _ => return None,
        };
        Some(ProviderHover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Sass.
#[cfg(feature = "lsp")]
pub struct SassLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: SassHoverProvider,
}
impl<V: Vfs> SassLanguageService<V> {
    /// Creates a new `SassLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: SassHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for SassLanguageService<V> {
    type Lang = SassLanguage;
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
            let language = SassLanguage::default();
            let parser = crate::parser::SassParser::new(&language);
            let lexer = crate::lexer::SassLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<SassLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
