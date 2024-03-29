#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{language::ProtobufLanguage, parser::element_type::ProtobufElementType};
use oak_core::{Range, tree::RedNode};
#[cfg(feature = "lsp")]
use {
    oak_hover::{Hover as ProviderHover, HoverProvider},
    oak_lsp::{service::LanguageService, types::Hover as LspHover},
    oak_vfs::Vfs,
    std::future::Future,
};
/// Hover provider implementation for Protobuf.
#[cfg(feature = "lsp")]
pub struct ProtobufHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<ProtobufLanguage> for ProtobufHoverProvider {
    fn hover(&self, node: &RedNode<ProtobufLanguage>, _range: Range<usize>) -> Option<ProviderHover> {
        let kind = node.green.kind;
        let contents = match kind {
            ProtobufElementType::MessageDef => "### Protobuf Message\nA message is a collection of fields. It's the primary way to define data structures in Protobuf.",
            ProtobufElementType::EnumDef => "### Protobuf Enum\nAn enum is a set of named integer constants.",
            ProtobufElementType::ServiceDef => "### Protobuf Service\nA service defines a set of RPC methods that can be called remotely.",
            _ => return None,
        };
        Some(ProviderHover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for Protobuf.
#[cfg(feature = "lsp")]
pub struct ProtobufLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: ProtobufHoverProvider,
}

impl<V: Vfs> ProtobufLanguageService<V> {
    /// Creates a new `ProtobufLanguageService` with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: ProtobufHoverProvider }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for ProtobufLanguageService<V> {
    type Lang = ProtobufLanguage;
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
            let language = ProtobufLanguage::default();
            let parser = crate::parser::ProtobufParser::new(&language);
            let lexer = crate::lexer::ProtobufLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<ProtobufLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
