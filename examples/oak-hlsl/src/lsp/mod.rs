#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{HlslLanguage, parser::element_type::HlslElementType};
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
/// Hover provider implementation for HLSL.
#[cfg(feature = "lsp")]
pub struct HlslHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<HlslLanguage> for HlslHoverProvider {
    fn hover(&self, node: &RedNode<HlslLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            HlslElementType::Struct => "### HLSL Struct\nDefines a custom data structure.",
            HlslElementType::Cbuffer => "### HLSL Constant Buffer\nDefines a buffer for shader constants.",
            HlslElementType::Technique => "### HLSL Technique\nDefines a rendering technique.",
            HlslElementType::Pass => "### HLSL Pass\nDefines a rendering pass within a technique.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for HLSL.
#[cfg(feature = "lsp")]
pub struct HlslLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: HlslHoverProvider,
}
impl<V: Vfs> HlslLanguageService<V> {
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: HlslHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::DocumentSession> LanguageService for HlslLanguageService<V> {
    type Lang = HlslLanguage;
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
            let language = HlslLanguage::default();
            let parser = crate::parser::HlslParser::new(&language);
            let lexer = crate::lexer::HlslLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<HlslLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
