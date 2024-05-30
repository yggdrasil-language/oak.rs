#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::{language::AplLanguage, parser::AplElementType};
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for APL.
#[cfg(feature = "lsp")]
pub struct AplHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<AplLanguage> for AplHoverProvider {
    fn hover(&self, node: &RedNode<AplLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            AplElementType::Assignment => "### APL Assignment\nAssigns a value to a variable.",
            AplElementType::ArrayLiteral => "### APL Array\nA collection of values.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for APL.
#[cfg(feature = "lsp")]
pub struct AplLanguageService<V: Vfs> {
    /// The virtual file system.
    pub vfs: V,
    /// The workspace manager.
    pub workspace: oak_lsp::workspace::WorkspaceManager,
    /// The hover provider.
    pub hover_provider: AplHoverProvider,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> AplLanguageService<V> {
    /// Creates a new `AplLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: AplHoverProvider }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for AplLanguageService<V> {
    type Lang = AplLanguage;
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
            let language = AplLanguage::default();
            let parser = crate::parser::AplParser::new(&language);
            let lexer = crate::lexer::AplLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<AplLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
