#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{OCamlLanguage, parser::element_type::OCamlElementType};
use oak_core::{Range, tree::RedNode};
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::HoverProvider,
    oak_lsp::{
        service::LanguageService,
        types::{Hover, Hover as LspHover},
    },
    oak_lsp::Vfs,
};
/// Hover provider implementation for OCaml.
#[cfg(feature = "lsp")]
pub struct OCamlHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<OCamlLanguage> for OCamlHoverProvider {
    fn hover(&self, node: &RedNode<OCamlLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            OCamlElementType::LetBinding => "### OCaml Let Binding\nDefines a value or function binding.",
            OCamlElementType::ModuleDef => "### OCaml Module\nDefines an OCaml module.",
            OCamlElementType::TypeDefinition => "### OCaml Type\nDefines a new type.",
            OCamlElementType::MatchExpr => "### OCaml Match\nPattern matching expression.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for OCaml.
#[cfg(feature = "lsp")]
pub struct OCamlLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: OCamlHoverProvider,
}
impl<V: Vfs> OCamlLanguageService<V> {
    /// Creates a new OCaml language service.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: OCamlHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for OCamlLanguageService<V> {
    type Lang = OCamlLanguage;
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
            let language = OCamlLanguage::default();
            let parser = crate::parser::OCamlParser::new(&language);
            let lexer = crate::lexer::OCamlLexer::new(&language);
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
