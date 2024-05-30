#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

use crate::{JsonLanguage, lexer::token_type::JsonTokenType};
use core::range::Range;
use oak_core::{TokenType, source::Source, tree::RedNode};
#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::Vfs,
    oak_lsp::service::LanguageService,
};

use crate::parser::element_type::JsonElementType;

/// Hover provider implementation for JSON.
#[cfg(feature = "lsp")]
pub struct JsonHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<JsonLanguage> for JsonHoverProvider {
    fn hover(&self, node: &RedNode<JsonLanguage>, _range: Range<usize>) -> Option<Hover> {
        let kind = node.green.kind;
        // Provide context-aware hover information
        let contents = match kind {
            JsonElementType::Object => "### JSON Object\nA collection of key-value pairs.",
            JsonElementType::Array => "### JSON Array\nAn ordered list of values.",
            JsonElementType::ObjectEntry => "### JSON Property\nA key-value pair in an object.",
            JsonElementType::StringLiteral => "### JSON String\nA sequence of Unicode characters.",
            JsonElementType::NumberLiteral => "### JSON Number\nA numeric value.",
            JsonElementType::BooleanLiteral => "### JSON Boolean\nA true or false value.",
            JsonElementType::NullLiteral => "### JSON Null\nRepresents the intentional absence of any value.",
            _ => return None,
        };
        Some(Hover { contents: contents.to_string(), range: Some(node.span()) })
    }
}
/// Language service implementation for JSON.
#[cfg(feature = "lsp")]
pub struct JsonLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: JsonHoverProvider,
}
impl<V: Vfs> JsonLanguageService<V> {
    /// Creates a new `JsonLanguageService`.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: JsonHoverProvider }
    }
    fn collect_definitions(&self, node: &RedNode<JsonLanguage>, name: &str, source: &V::Source, uri: &str, definitions: &mut Vec<oak_lsp::LocationRange>) {
        use oak_core::{
            language::{ElementType, UniversalElementRole, UniversalTokenRole},
            tree::RedTree,
        };
        // In JSON, every ObjectEntry can be considered a definition if its key matches
        if ElementType::is_universal(&node.green.kind, UniversalElementRole::Statement) && node.green.kind == JsonElementType::ObjectEntry {
            for child in node.children() {
                if let RedTree::Leaf(leaf) = child {
                    // Keys are Name (if BareKey) or Literal (if StringLiteral)
                    // But for navigation, we treat the key as the identifier
                    if TokenType::is_universal(&leaf.kind, UniversalTokenRole::Name) || leaf.kind == JsonTokenType::StringLiteral {
                        let text = source.get_text_in(leaf.span.clone());
                        // Strip quotes for string literals
                        let key_name = if leaf.kind == JsonTokenType::StringLiteral { text.trim_matches('"') } else { &text };
                        if key_name == name {
                            definitions.push(oak_lsp::LocationRange { uri: uri.to_string().into(), range: leaf.span });
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
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for JsonLanguageService<V> {
    type Lang = JsonLanguage;
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
            let language = JsonLanguage::default();
            let parser = crate::parser::JsonParser::new(&language);
            let lexer = crate::lexer::JsonLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn definition<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Vec<oak_lsp::LocationRange>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let name = self
                .with_root(&uri, |root| {
                    let source = self.vfs().request_source(&uri)?;
                    let leaf = root.leaf_at_offset(range.start)?;
                    let text = source.get_text_in(leaf.span.clone());
                    Some(text.trim_matches('"').to_string())
                })
                .await
                .flatten();
            let Some(name) = name
            else {
                return vec![];
            };
            let mut all_definitions = Vec::new();
            let files = self.list_all_files(&uri).await;
            for file_uri in files {
                if let Some(defs) = self
                    .with_root(&file_uri, |file_root| {
                        let mut defs = Vec::new();
                        if let Some(file_source) = self.vfs().request_source(&file_uri) {
                            self.collect_definitions(&file_root, &name, &file_source, &file_uri, &mut defs);
                        }
                        defs
                    })
                    .await
                {
                    all_definitions.extend(defs);
                }
            }
            all_definitions
        }
    }
    fn references<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Vec<oak_lsp::LocationRange>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let name = self
                .with_root(&uri, |root| {
                    let source = self.vfs().request_source(&uri)?;
                    let leaf = root.leaf_at_offset(range.start)?;
                    let text = source.get_text_in(leaf.span.clone());
                    Some(text.trim_matches('"').to_string())
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
                        if let Some(file_source) = self.vfs().request_source(&file_uri) {
                            self.collect_definitions(&file_root, &name, &file_source, &file_uri, &mut refs);
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
                // Ensure new name is quoted if the original was quoted
                let Some(source) = self.vfs().request_source(&r.uri)
                else {
                    continue;
                };
                let old_text = source.get_text_in(r.range.clone());
                let formatted_new_name = if old_text.starts_with('"') { format!("\"{}\"", new_name) } else { new_name.clone() };
                changes.entry(r.uri.to_string()).or_insert_with(Vec::new).push(oak_lsp::TextEdit { range: r.range, new_text: formatted_new_name });
            }
            Some(oak_lsp::WorkspaceEdit { changes })
        }
    }
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<JsonLanguage> + Send + Sync)> {
        Some(&self.hover_provider)
    }
}
