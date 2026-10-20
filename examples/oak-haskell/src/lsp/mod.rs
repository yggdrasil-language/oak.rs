#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{HaskellLanguage, lexer::token_type::HaskellTokenType, parser::element_type::HaskellElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::service::LanguageService,
    oak_vfs::Vfs,
};
/// Hover provider implementation for Haskell.
#[cfg(feature = "lsp")]
pub struct HaskellHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<HaskellLanguage> for HaskellHoverProvider {
    fn hover(&self, node: &RedNode<HaskellLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            HaskellElementType::Function => "### Haskell Function\nDefines a transformation from inputs to outputs.",
            HaskellElementType::DataDeclaration => "### Haskell Data Type\nDefines a new algebraic data type.",
            HaskellElementType::ModuleDeclaration => "### Haskell Module\nOrganizes Haskell code into namespaces.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Haskell.
#[cfg(feature = "lsp")]
pub struct HaskellLanguageService<V: Vfs> {
    /// The virtual file system.
    vfs: V,
    /// The workspace manager.
    workspace: oak_lsp::workspace::WorkspaceManager,
    /// The hover provider.
    hover_provider: HaskellHoverProvider,
}
impl<V: Vfs> HaskellLanguageService<V> {
    /// Creates a new `HaskellLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: HaskellHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for HaskellLanguageService<V> {
    type Lang = HaskellLanguage;
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
            let language = HaskellLanguage::default();
            let parser = crate::parser::HaskellParser::new(&language);
            let lexer = crate::lexer::HaskellLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<HaskellLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
