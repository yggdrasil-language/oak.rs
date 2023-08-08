#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{MatlabLanguage, parser::element_type::MatlabElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {oak_hover::HoverProvider, oak_lsp::service::LanguageService, oak_vfs::Vfs, std::future::Future};
/// Hover provider implementation for MATLAB.
#[cfg(feature = "lsp")]
pub struct MatlabHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<MatlabLanguage> for MatlabHoverProvider {
    fn hover(&self, node: &RedNode<MatlabLanguage>, _range: Range<usize>) -> Option<oak_hover::Hover> {
        let kind = node.green.kind;
        let contents = match kind {
            MatlabElementType::Root => "### MATLAB Root\nTop-level expression container.",
            MatlabElementType::Call => "### MATLAB Call\nFunction or indexing call `f(…)`.",
            MatlabElementType::Array => "### MATLAB Array\nMatrix / row literal `[…]`.",
            MatlabElementType::CellArray => "### MATLAB CellArray\nCell array literal `{…}`.",
            MatlabElementType::MemberAccess => "### MATLAB Member\nMember / package access `a.b`.",
            MatlabElementType::BinaryExpr => "### MATLAB BinaryExpr\nBinary operator application.",
            MatlabElementType::Symbol => "### MATLAB Symbol\nIdentifier / name.",
            MatlabElementType::Literal => "### MATLAB Literal\nNumber or string literal.",
            MatlabElementType::CommandStmt => "### MATLAB Command\nCommand syntax `name arg1 arg2` (no parentheses).",
            MatlabElementType::GlobalStmt => "### MATLAB Global\n`global` name declaration.",
            MatlabElementType::PersistentStmt => "### MATLAB Persistent\n`persistent` name declaration.",
            _ => return None,
        };
        Some(oak_hover::Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for MATLAB.
#[cfg(feature = "lsp")]
pub struct MatlabLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: MatlabHoverProvider,
}
impl<V: Vfs> MatlabLanguageService<V> {
    /// Creates a new `MatlabLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: MatlabHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for MatlabLanguageService<V> {
    type Lang = MatlabLanguage;
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
            let language = MatlabLanguage::default();
            let parser = crate::parser::MatlabParser::new(&language);
            let lexer = crate::lexer::MatlabLexer::new(&language);
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
