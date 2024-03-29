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
use crate::{TclLanguage, parser::element_type::TclElementType};
use core::range::Range;
use oak_core::tree::RedNode;
/// Tcl hover provider implementation.
#[cfg(feature = "lsp")]
pub struct TclHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<TclLanguage> for TclHoverProvider {
    fn hover(&self, node: &RedNode<TclLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            TclElementType::Proc => "### Tcl Procedure\nDefines a command.",
            TclElementType::Set => "### Tcl Set\nSets the value of a variable.",
            _ => return None,
        };

        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}

/// Tcl language service implementation.
#[cfg(feature = "lsp")]
pub struct TclLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: TclHoverProvider,
}
impl<V: Vfs> TclLanguageService<V> {
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: TclHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for TclLanguageService<V> {
    type Lang = TclLanguage;
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
            let language = TclLanguage::default();
            let parser = crate::parser::TclParser::new(&language);
            let lexer = crate::lexer::TclLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<TclLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
