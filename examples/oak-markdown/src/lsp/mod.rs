#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{language::MarkdownLanguage, parser::element_type::MarkdownElementType};
use core::range::Range;
use oak_core::{ElementRole, ElementType, Source, TokenType, tree::RedNode};
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover as HoverInfo, HoverProvider},
    oak_lsp::{service::LanguageService, types::Hover as LspHover},
    oak_vfs::Vfs,
};
/// Hover provider for Markdown.
#[cfg(feature = "lsp")]
pub struct MarkdownHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<MarkdownLanguage> for MarkdownHoverProvider {
    fn hover(&self, node: &RedNode<'_, MarkdownLanguage>, _range: Range<usize>) -> Option<HoverInfo> {
        let kind = node.green.kind;
        let contents = match kind {
            MarkdownElementType::Heading1 => "### Heading 1\nTop-level heading.",
            MarkdownElementType::Heading2 => "### Heading 2\nSecond-level heading.",
            MarkdownElementType::Link => "### Markdown Link\nLink to an external resource or internal anchor.",
            MarkdownElementType::CodeBlock => "### Code Block\nA block of source code.",
            MarkdownElementType::Strong => "### Strong Emphasis\nBold text.",
            MarkdownElementType::Emphasis => "### Emphasis\nItalic text.",
            _ => return None,
        };
        Some(HoverInfo { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service for Markdown providing LSP features.
#[cfg(feature = "lsp")]
pub struct MarkdownLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: MarkdownHoverProvider,
}
impl<V: Vfs> MarkdownLanguageService<V> {
    /// Creates a new MarkdownLanguageService with the given VFS.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::new(), hover_provider: MarkdownHoverProvider }
    }
    fn collect_definitions(&self, node: &RedNode<MarkdownLanguage>, name: &str, source: &str, uri: &str, definitions: &mut Vec<oak_lsp::LocationRange>) {
        use oak_core::{language::UniversalElementRole, tree::RedTree};
        let role = ElementType::role(&node.green.kind);
        if ElementRole::universal(&role) == UniversalElementRole::Definition {
            for child in node.children() {
                if let RedTree::Leaf(leaf) = child {
                    if TokenType::is_universal(&leaf.kind, oak_core::language::UniversalTokenRole::Name) {
                        if &source[leaf.span.clone()] == name {
                            definitions.push(oak_lsp::LocationRange { uri: uri.into(), range: leaf.span });
                            return;
                        }
                    }
                }
            }
        }
        for child in node.children() {
            if let RedTree::Node(child_node) = child {
                self.collect_definitions(&child_node, name, source, uri, definitions);
            }
        }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_vfs::WritableVfs> LanguageService for MarkdownLanguageService<V> {
    type Lang = MarkdownLanguage;
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
            let language = MarkdownLanguage::github_flavored();
            let parser = crate::parser::MarkdownParser::new(&language);
            let lexer = crate::lexer::MarkdownLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn definition<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Vec<oak_lsp::LocationRange>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            self.with_root(&uri, |root| {
                let source = self.vfs().get_source(&uri)?;
                let leaf = root.leaf_at_offset(range.start)?;
                if !TokenType::is_universal(&leaf.kind, oak_core::language::UniversalTokenRole::Name) {
                    return None;
                }
                let name = source.get_text_in(leaf.span.clone());
                let mut definitions = Vec::new();
                let full_text = source.get_text_in(Range { start: 0, end: source.length() });
                self.collect_definitions(&root, &name, &full_text, &uri, &mut definitions);
                Some(definitions)
            })
            .await
            .flatten()
            .unwrap_or_default()
        }
    }
    fn references<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Vec<oak_lsp::LocationRange>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let name = self
                .with_root(&uri, |root| {
                    let source = self.vfs().get_source(&uri)?;
                    let leaf = root.leaf_at_offset(range.start)?;
                    if !TokenType::is_universal(&leaf.kind, oak_core::language::UniversalTokenRole::Name) {
                        return None;
                    }
                    Some(source.get_text_in(leaf.span.clone()).to_string())
                })
                .await
                .flatten();
            let Some(name) = name
            else {
                return vec![];
            };
            let mut all_refs = Vec::new();
            let files = self.list_all_files(&uri).await;
            for file_uri in files {
                if let Some(refs) = self
                    .with_root(&file_uri, |file_root| {
                        let mut refs = Vec::new();
                        if let Some(file_source) = self.vfs().get_source(&file_uri) {
                            let full_text = file_source.get_text_in(Range { start: 0, end: file_source.length() });
                            let found = oak_navigation::SimpleReferenceFinder::find(&file_root, &name, &full_text, file_uri.clone());
                            refs.extend(found.into_iter().map(|l| oak_lsp::LocationRange { uri: l.uri, range: l.range }));
                        }
                        refs
                    })
                    .await
                {
                    all_refs.extend(refs);
                }
            }
            all_refs
        }
    }
    fn rename<'a>(&'a self, uri: &'a str, range: Range<usize>, new_name: String) -> impl Future<Output = Option<oak_lsp::WorkspaceEdit>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let refs = self.references(&uri, range).await;
            if refs.is_empty() {
                return None;
            }
            let mut changes = std::collections::HashMap::new();
            for r in refs {
                changes.entry(r.uri.to_string()).or_insert_with(Vec::new).push(oak_lsp::TextEdit { range: r.range, new_text: new_name.clone() });
            }
            Some(oak_lsp::WorkspaceEdit { changes })
        }
    }
    fn hover(&self, uri: &str, range: Range<usize>) -> impl Future<Output = Option<LspHover>> + Send + '_ {
        let uri = uri.to_string();
        async move { self.with_root(&uri, |root| self.hover_provider.hover(&root, range).map(|h| LspHover { contents: h.contents, range: h.range })).await.flatten() }
    }
}
