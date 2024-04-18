#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{ActionScriptLanguage, parser::ActionScriptElementType};
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
/// Hover provider implementation for ActionScript.
#[cfg(feature = "lsp")]
pub struct ActionScriptHoverProvider;

#[cfg(feature = "lsp")]
impl HoverProvider<ActionScriptLanguage> for ActionScriptHoverProvider {
    fn hover(&self, node: &RedNode<ActionScriptLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;

        let contents = match kind {
            ActionScriptElementType::Class => "### ActionScript Class\nDefines a blueprint for objects.",
            ActionScriptElementType::Interface => "### ActionScript Interface\nDefines a contract for classes.",
            ActionScriptElementType::Function => "### ActionScript Function\nDefines a block of code that performs a task.",
            _ => return None,
        };

        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}

#[cfg(feature = "lsp")]
/// Language service implementation for ActionScript.
#[cfg(feature = "lsp")]
pub struct ActionScriptLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: ActionScriptHoverProvider,
}

#[cfg(feature = "lsp")]
impl<V: Vfs> ActionScriptLanguageService<V> {
    /// Creates a new `ActionScriptLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: ActionScriptHoverProvider }
    }
}

#[cfg(feature = "lsp")]
impl<V: Vfs + Send + Sync + 'static + oak_vfs::DocumentSession> LanguageService for ActionScriptLanguageService<V> {
    type Lang = ActionScriptLanguage;
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
            let language = ActionScriptLanguage::default();
            let parser = crate::parser::ActionScriptParser::new(&language);
            let lexer = crate::lexer::ActionScriptLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }

    fn hover_provider(&self) -> Option<&(dyn HoverProvider<ActionScriptLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }

    fn completion(&self, _uri: &str, _offset: usize) -> impl Future<Output = Vec<oak_lsp::types::CompletionItem>> + Send + '_ {
        async move { vec![] }
    }
}
