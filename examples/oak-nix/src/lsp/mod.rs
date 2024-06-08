#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{NixLanguage, parser::element_type::NixElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {oak_hover::HoverProvider, oak_lsp::Vfs, oak_lsp::service::LanguageService, std::future::Future};
/// Hover provider implementation for Nix.
#[cfg(feature = "lsp")]
pub struct NixHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<NixLanguage> for NixHoverProvider {
    fn hover(&self, node: &RedNode<NixLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            NixElementType::Let => "### Nix Let expression\nStarts a let-binding block.",
            NixElementType::With => "### Nix With expression\nBrings a set's attributes into scope.",
            NixElementType::Import => "### Nix Import\nImports another Nix expression.",
            NixElementType::Root => "### Nix Expression\nThe root of a Nix configuration.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Nix.
#[cfg(feature = "lsp")]
pub struct NixLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: NixHoverProvider,
}
impl<V: Vfs> NixLanguageService<V> {
    /// Creates a new Nix language service with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: NixHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for NixLanguageService<V> {
    type Lang = NixLanguage;
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
            let language = NixLanguage::default();
            let parser = crate::parser::NixParser::new(&language);
            let lexer = crate::lexer::NixLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Option<oak_lsp::types::Hover>> + Send + '_ {
        let uri = uri.to_string();
        async move {
            let res = self.with_root(&uri, |root| self.hover_provider.hover(&root, range)).await.flatten();
            res.map(|h| oak_lsp::types::Hover { contents: h.contents, range: h.range })
        }
    }
}
