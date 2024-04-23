#![doc = include_str!("readme.md")]
pub mod formatter;
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{PerlLanguage, parser::element_type::PerlElementType};
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    oak_hover::{Hover as ProviderHover, HoverProvider},
    oak_lsp::{service::LanguageService, types::Hover as LspHover},
    oak_lsp::Vfs,
};
/// Hover provider implementation for Perl.
#[cfg(feature = "lsp")]
pub struct PerlHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<PerlLanguage> for PerlHoverProvider {
    fn hover(&self, node: &RedNode<PerlLanguage>, _range: Range<usize>) -> Option<ProviderHover> {
        let kind = node.green.kind;
        let contents = match kind {
            PerlElementType::SubroutineDeclaration => "### Perl Subroutine\nA named block of code that can be called.",
            PerlElementType::PackageDeclaration => "### Perl Package\nDefines a namespace for the code.",
            PerlElementType::VariableDeclaration => "### Perl Variable\nA variable declared with `my`, `our`, or `local`.",
            PerlElementType::IfStatement => "### If Statement\nConditional execution block.",
            _ => return None,
        };
        Some(ProviderHover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Perl.
///
/// This service provides LSP features like hover, completion, and diagnostics
/// for the Perl language, utilizing the VFS for file management.
#[cfg(feature = "lsp")]
pub struct PerlLanguageService<V: Vfs> {
    /// Virtual file system.
    vfs: V,
    /// Workspace manager.
    workspace: oak_lsp::workspace::WorkspaceManager,
    /// Hover provider.
    hover_provider: PerlHoverProvider,
}
impl<V: Vfs> PerlLanguageService<V> {
    /// Creates a new `PerlLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: PerlHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for PerlLanguageService<V> {
    type Lang = PerlLanguage;
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
            let language = PerlLanguage::default();
            let parser = crate::parser::PerlParser::new(&language);
            let lexer = crate::lexer::PerlLexer::new(&language);
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
    fn completion(&self, _uri: &str, _offset: usize) -> impl futures::Future<Output = Vec<oak_lsp::types::CompletionItem>> + Send + '_ {
        async move { vec![] }
    }
}
