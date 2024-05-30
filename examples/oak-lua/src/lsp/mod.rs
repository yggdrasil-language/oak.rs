#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::language::LuaLanguage;
use core::range::Range;
use oak_core::tree::RedNode;
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};
/// Hover provider implementation for Lua.
#[cfg(feature = "lsp")]
pub struct LuaHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<LuaLanguage> for LuaHoverProvider {
    fn hover(&self, node: &RedNode<LuaLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        // This is a simplified example
        let contents = match format!("{:?}", kind).as_str() {
            "Function" => "### Lua Function\nA block of code that performs a specific task.",
            "LocalDeclaration" => "### Local Variable\nA variable with local scope.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Lua.
#[cfg(feature = "lsp")]
pub struct LuaLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: LuaHoverProvider,
}
#[cfg(feature = "lsp")]
impl<V: Vfs> LuaLanguageService<V> {
    /// Creates a new `LuaLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: LuaHoverProvider }
    }
}
#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for LuaLanguageService<V> {
    type Lang = LuaLanguage;
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
            let language = LuaLanguage::default();
            let parser = crate::parser::LuaParser::new(&language);
            let lexer = crate::lexer::LuaLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<LuaLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
