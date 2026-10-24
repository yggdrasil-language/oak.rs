#![doc = include_str!("readme.md")]
use crate::YamlLanguage;
use oak_core::tree::RedNode;
/// Highlighter module.
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {oak_lsp::Vfs, oak_lsp::service::LanguageService, std::future::Future};
/// MCP module.
/// Formatter module.
#[cfg(feature = "lsp")]
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
/// Language service implementation for YAML.
#[cfg(feature = "lsp")]
pub struct YamlLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> YamlLanguageService<V> {
    /// Creates a new `YamlLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default() }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for YamlLanguageService<V> {
    type Lang = YamlLanguage;
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
            let language = YamlLanguage::default();
            let parser = crate::parser::YamlParser::new(&language);
            let lexer = crate::lexer::YamlLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
}
