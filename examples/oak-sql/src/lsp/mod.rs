#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    crate::language::SqlLanguage,
    core::range::Range,
    futures::Future,
    oak_core::tree::RedNode,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};

#[cfg(feature = "oak-pretty-print")]
pub mod formatter;

#[cfg(feature = "lsp")]
/// A hover provider for SQL.
pub struct SqlHoverProvider;

#[cfg(feature = "lsp")]
impl HoverProvider<SqlLanguage> for SqlHoverProvider {
    fn hover(&self, node: &RedNode<SqlLanguage>, _range: Range<usize>) -> Option<Hover> {
        let _kind = node.green.kind;
        None
    }
}

#[cfg(feature = "lsp")]
/// Language service implementation for SQL.
pub struct SqlLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: SqlHoverProvider,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> SqlLanguageService<V> {
    /// Creates a new SQL language service.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: SqlHoverProvider }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for SqlLanguageService<V> {
    type Lang = SqlLanguage;
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
            let language = SqlLanguage::default();
            let parser = crate::parser::SqlParser::new(&language);
            let lexer = crate::lexer::SqlLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<SqlLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
