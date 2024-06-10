#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{language::NimLanguage, parser::element_type::NimElementType};
use dashmap::DashMap;
use oak_core::{Range, parser::session::ParseSession, tree::RedNode};
#[cfg(feature = "lsp")]
use oak_hover::HoverProvider;
#[cfg(feature = "lsp")]
use oak_lsp::Vfs;
#[cfg(feature = "lsp")]
use oak_lsp::service::LanguageService;
use std::{future::Future, sync::Arc};
/// Hover provider implementation for Nim.
#[cfg(feature = "lsp")]
pub struct NimHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<NimLanguage> for NimHoverProvider {
    fn hover(&self, node: &RedNode<'_, NimLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            NimElementType::ProcKeyword => "### Nim Procedure\nDefines a callable procedure in Nim.",
            NimElementType::TypeKeyword => "### Nim Type Definition\nDefines a new type or type alias.",
            NimElementType::ConstKeyword => "### Nim Constant\nDefines a compile-time constant.",
            NimElementType::Root => "### Nim Module\nThe root of a Nim module.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Nim.
#[cfg(feature = "lsp")]
pub struct NimLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: NimHoverProvider,
    _cache: Arc<DashMap<String, ParseSession<NimLanguage>>>,
}
impl<V: Vfs> NimLanguageService<V> {
    /// Creates a new Nim language service
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: NimHoverProvider, _cache: Arc::new(DashMap::new()) }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for NimLanguageService<V> {
    type Lang = NimLanguage;
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
            let language = NimLanguage::default();
            let parser = crate::parser::NimParser::new(&language);
            let lexer = crate::lexer::NimLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Option<oak_lsp::types::Hover>> + Send + '_ {
        let uri = uri.to_string();
        async move {
            let hover = self.with_root(&uri, |root| self.hover_provider.hover(&root, range)).await.flatten()?;
            Some(oak_lsp::types::Hover { contents: hover.contents, range: hover.range })
        }
    }
}
