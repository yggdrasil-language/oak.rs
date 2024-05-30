#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{CoqLanguage, parser::element_type::CoqElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};
/// Hover provider implementation for Coq.
#[cfg(feature = "lsp")]
pub struct CoqHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<CoqLanguage> for CoqHoverProvider {
    fn hover(&self, node: &RedNode<CoqLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            CoqElementType::Declaration => "### Coq Declaration\nDefines a new term, theorem, or inductive type.",
            CoqElementType::Statement => "### Coq Statement\nA vernacular command or tactic.",
            CoqElementType::Expression => "### Coq Expression\nA Coq term or type.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Coq.
#[cfg(feature = "lsp")]
pub struct CoqLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: CoqHoverProvider,
}
impl<V: Vfs> CoqLanguageService<V> {
    /// Creates a new `CoqLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: CoqHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for CoqLanguageService<V> {
    type Lang = CoqLanguage;
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
            let language = CoqLanguage::default();
            let parser = crate::parser::CoqParser::new(&language);
            let lexer = crate::lexer::CoqLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<CoqLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
