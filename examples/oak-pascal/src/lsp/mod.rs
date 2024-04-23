#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{PascalElementType, PascalLanguage};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use oak_hover::HoverProvider;
#[cfg(feature = "lsp")]
use oak_lsp::{service::LanguageService, types::Hover};
#[cfg(feature = "lsp")]
use oak_lsp::Vfs;
/// Hover provider implementation for Pascal.
#[cfg(feature = "lsp")]
pub struct PascalHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<PascalLanguage> for PascalHoverProvider {
    fn hover(&self, node: &RedNode<PascalLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            PascalElementType::Program => "### Pascal Program\nThe main entry point of the application.",
            PascalElementType::Procedure => "### Pascal Procedure\nA subprogram that performs a specific task.",
            PascalElementType::Function => "### Pascal Function\nA subprogram that returns a value.",
            PascalElementType::VarSection => "### Variable Declaration Section\nDeclares variables for use in the program.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Pascal.
#[cfg(feature = "lsp")]
pub struct PascalLanguageService<V: Vfs> {
    /// The virtual file system.
    pub vfs: V,
    /// The workspace manager.
    pub workspace: oak_lsp::workspace::WorkspaceManager,
    /// The hover provider.
    pub hover_provider: PascalHoverProvider,
}
impl<V: Vfs> PascalLanguageService<V> {
    /// Creates a new Pascal language service.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: PascalHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for PascalLanguageService<V> {
    type Lang = PascalLanguage;
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
            let language = PascalLanguage::default();
            let parser = crate::parser::PascalParser::new(&language);
            let lexer = crate::lexer::PascalLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl std::future::Future<Output = Option<Hover>> + Send + '_ {
        let uri = uri.to_string();
        async move { self.with_root(&uri, |root| self.hover_provider.hover(&root, range).map(|h| Hover { contents: h.contents, range: h.range })).await.flatten() }
    }
}
