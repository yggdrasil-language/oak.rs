#![doc = include_str!("readme.md")]

/// Formatter module.
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::JasmLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::service::LanguageService,
    oak_vfs::Vfs,
};
#[cfg(feature = "lsp")]
/// Hover provider for Jasm.
pub struct JasmHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<JasmLanguage> for JasmHoverProvider {
    fn hover(&self, node: &RedNode<JasmLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        let contents = format!("### JASM Node\nKind: {:?}", kind);
        Some(Hover { contents, range: Some(node.span()) })
    }
}
#[cfg(feature = "lsp")]
/// Language service for Jasm.
pub struct JasmLanguageService<V: Vfs> {
    /// The virtual file system.
    pub vfs: V,
    /// The workspace manager.
    pub workspace: oak_lsp::workspace::WorkspaceManager,
    /// The hover provider.
    pub hover_provider: JasmHoverProvider,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> JasmLanguageService<V> {
    /// Creates a new instance of `JasmLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: JasmHoverProvider }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_vfs::DocumentSession> LanguageService for JasmLanguageService<V> {
    type Lang = JasmLanguage;
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
            let language = JasmLanguage::default();
            let parser = crate::parser::JasmParser::new(&language);
            let lexer = crate::lexer::JasmLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<JasmLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
