#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::service::LanguageService,
    oak_vfs::Vfs,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::{AdaLanguage, parser::AdaElementType};
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for Ada.
#[cfg(feature = "lsp")]
pub struct AdaHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<AdaLanguage> for AdaHoverProvider {
    fn hover(&self, node: &RedNode<AdaLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            AdaElementType::SubprogramDeclaration => "### Ada Subprogram\nDefines a procedure or function.",
            AdaElementType::PackageDeclaration => "### Ada Package\nDefines a module of code.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Ada.
#[cfg(feature = "lsp")]
pub struct AdaLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: AdaHoverProvider,
}
impl<V: Vfs> AdaLanguageService<V> {
    /// Creates a new `AdaLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: AdaHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for AdaLanguageService<V> {
    type Lang = AdaLanguage;
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
            let language = AdaLanguage::default();
            let parser = crate::parser::AdaParser::new(&language);
            let lexer = crate::lexer::AdaLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<AdaLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
