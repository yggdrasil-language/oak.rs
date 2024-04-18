#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    oak_hover::{Hover as ProviderHover, HoverProvider},
    oak_lsp::{service::LanguageService, types::Hover as LspHover},
    oak_vfs::Vfs,
    std::future::Future,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
use crate::{ScalaLanguage, lexer::token_type::ScalaTokenType, parser::element_type::ScalaElementType};
use core::range::Range;
use oak_core::tree::RedNode;
/// Hover provider implementation for Scala.
#[cfg(feature = "lsp")]
pub struct ScalaHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<ScalaLanguage> for ScalaHoverProvider {
    fn hover(&self, node: &RedNode<ScalaLanguage>, _range: Range<usize>) -> Option<ProviderHover> {
        let kind = node.green.kind;
        let contents = match kind {
            ScalaElementType::SourceFile => "### Scala Source File\nEntry point for Scala code.",
            _ => return None,
        };
        Some(ProviderHover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Scala.
#[cfg(feature = "lsp")]
pub struct ScalaLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: ScalaHoverProvider,
}
impl<V: Vfs> ScalaLanguageService<V> {
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: ScalaHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::DocumentSession> LanguageService for ScalaLanguageService<V> {
    type Lang = ScalaLanguage;
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
            let language = ScalaLanguage::default();
            let parser = crate::parser::ScalaParser::new(&language);
            let lexer = crate::lexer::ScalaLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<ScalaLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
