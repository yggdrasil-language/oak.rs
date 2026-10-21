#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{PowerShellElementType, PowerShellLanguage};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    oak_hover::{Hover as ProviderHover, HoverProvider},
    oak_lsp::{service::LanguageService, types::Hover as LspHover},
    oak_lsp::Vfs,
};
/// Hover provider implementation for PowerShell.
#[cfg(feature = "lsp")]
pub struct PowerShellHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<PowerShellLanguage> for PowerShellHoverProvider {
    fn hover(&self, node: &RedNode<PowerShellLanguage>, _range: Range<usize>) -> Option<ProviderHover> {
        let kind = node.green.kind;
        let contents = match kind {
            PowerShellElementType::FunctionDef => "### PowerShell Function\nA named block of code that performs a specific task.",
            PowerShellElementType::ClassDef => "### PowerShell Class\nA blueprint for creating objects in PowerShell.",
            PowerShellElementType::IfStatement => "### PowerShell If Statement\nA conditional statement that executes a block of code if a condition is true.",
            _ => return None,
        };
        Some(ProviderHover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for PowerShell.
#[cfg(feature = "lsp")]
pub struct PowerShellLanguageService<V: Vfs> {
    /// The virtual file system.
    pub vfs: V,
    /// The workspace manager.
    pub workspace: oak_lsp::workspace::WorkspaceManager,
    /// The hover provider.
    pub hover_provider: PowerShellHoverProvider,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> PowerShellLanguageService<V> {
    /// Creates a new `PowerShellLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: PowerShellHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for PowerShellLanguageService<V> {
    type Lang = PowerShellLanguage;
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
            let language = PowerShellLanguage::default();
            let parser = crate::parser::PowerShellParser::new(&language);
            let lexer = crate::lexer::PowerShellLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl futures::Future<Output = Option<LspHover>> + Send + '_ {
        let uri = uri.to_string();
        async move { self.with_root(&uri, |root| self.hover_provider.hover(&root, range).map(|h| LspHover { contents: h.contents, range: h.range })).await.flatten() }
    }
}
