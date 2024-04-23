#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{MsilLanguage, parser::element_type::MsilElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    oak_hover::HoverProvider,
    oak_lsp::{service::LanguageService, types::Hover},
    oak_lsp::Vfs,
    std::future::Future,
};
/// Hover provider implementation for MSIL.
#[cfg(feature = "lsp")]
pub struct MsilHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<MsilLanguage> for MsilHoverProvider {
    fn hover(&self, node: &RedNode<MsilLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            MsilElementType::Method => "### MSIL Method\nDefines a method in MSIL.",
            MsilElementType::Class => "### MSIL Class\nDefines a class in MSIL.",
            MsilElementType::Assembly => "### MSIL Assembly\nDefines an assembly in MSIL.",
            MsilElementType::Instruction => "### MSIL Instruction\nA single MSIL instruction.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for MSIL.
#[cfg(feature = "lsp")]
pub struct MsilLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: MsilHoverProvider,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> MsilLanguageService<V> {
    /// Create new MSIL language service
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: MsilHoverProvider }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for MsilLanguageService<V> {
    type Lang = MsilLanguage;
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
            let language = MsilLanguage::default();
            let parser = crate::parser::MsilParser::new(&language);
            let lexer = crate::lexer::MsilLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Option<Hover>> + Send + '_ {
        let uri = uri.to_string();
        async move { self.with_root(&uri, |root| self.hover_provider.hover(&root, range).map(|h| Hover { contents: h.contents, range: h.range })).await.flatten() }
    }
}
