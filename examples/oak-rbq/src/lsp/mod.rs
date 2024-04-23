#![doc = include_str!("readme.md")]
#[cfg(feature = "oak-highlight")]
pub mod highlighter;

#[cfg(feature = "lsp")]
use {
    futures::Future,
    oak_hover::{Hover, HoverProvider},
    oak_lsp::service::LanguageService,
    oak_lsp::Vfs,
};
#[cfg(feature = "oak-pretty-print")]
pub mod formatter;
pub mod semantic_tokens;
use self::semantic_tokens::RbqSemanticTokensProvider;
use crate::{language::RbqLanguage, lexer::token_type::RbqTokenType, parser::element_type::RbqElementType};
use core::range::Range;
use oak_core::{Parser, source::Source, tree::RedNode};
use oak_folding::{FoldingProvider, FoldingRange, FoldingRangeKind};
/// RBQ hover provider.
#[cfg(feature = "lsp")]
pub struct RbqHoverProvider;
#[cfg(feature = "lsp")]
impl HoverProvider<RbqLanguage> for RbqHoverProvider {
    fn hover(&self, node: &RedNode<RbqLanguage>, _range: Range<usize>) -> Option<Hover> {
        let _kind = node.element_type();
        None
    }
}
/// RBQ folding provider.
pub struct RbqFoldingProvider;
impl FoldingProvider<RbqLanguage> for RbqFoldingProvider {
    fn folding_ranges(&self, root: &RedNode<RbqLanguage>) -> Vec<FoldingRange> {
        let mut ranges = Vec::new();
        self.collect_folding_ranges(root, &mut ranges);
        ranges
    }
}
impl RbqFoldingProvider {
    fn collect_folding_ranges(&self, node: &RedNode<RbqLanguage>, ranges: &mut Vec<FoldingRange>) {
        match node.element_type() {
            RbqElementType::NAMESPACE_DEFINITION | RbqElementType::STRUCT_DEFINITION | RbqElementType::ENUM_DEFINITION => {
                ranges.push(FoldingRange { range: node.span(), kind: None });
            }
            RbqElementType::Comment | RbqElementType::BlockComment => {
                ranges.push(FoldingRange { range: node.span(), kind: Some(FoldingRangeKind::Comment) });
            }
            _ => {}
        }
        for child in node.children() {
            if let oak_core::tree::RedTree::Node(n) = child {
                self.collect_folding_ranges(&n, ranges);
            }
        }
    }
}
use dashmap::DashMap;
use oak_core::parser::ParseSession;
/// RBQ language service.
#[cfg(feature = "lsp")]
pub struct RbqLanguageService<V: Vfs> {
    vfs: V,
    workspace: oak_lsp::workspace::WorkspaceManager,
    hover_provider: RbqHoverProvider,
    folding_provider: RbqFoldingProvider,
    semantic_tokens_provider: RbqSemanticTokensProvider,
    caches: DashMap<String, ParseSession<RbqLanguage>>,
}

impl<V: Vfs> RbqLanguageService<V> {
    /// Creates a new RBQ language service.
    pub fn new(vfs: V) -> Self {
        Self { vfs, workspace: oak_lsp::workspace::WorkspaceManager::default(), hover_provider: RbqHoverProvider, folding_provider: RbqFoldingProvider, semantic_tokens_provider: RbqSemanticTokensProvider, caches: DashMap::new() }
    }
}
impl<V: Vfs + Send + Sync + 'static + oak_lsp::DocumentSession> LanguageService for RbqLanguageService<V> {
    type Lang = RbqLanguage;
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
            let language = RbqLanguage::default();
            let parser = crate::parser::RbqParser::new(&language);
            let lexer = crate::lexer::RbqLexer::new(&language);
            let mut cache = oak_core::parser::session::ParseSession::<Self::Lang>::default();
            let parse_out = oak_core::parser::parse(&parser, &lexer, &source, &[], &mut cache);
            let green = parse_out.result.ok()?;
            Some(f(RedNode::new(green, 0)))
        }
    }
    fn folding_ranges(&self, uri: &str) -> impl Future<Output = Vec<oak_lsp::FoldingRange>> + Send + '_ {
        let uri = uri.to_string();
        async move {
            let source = self.vfs().request_source(&uri).unwrap(); // Handle error properly in real impl
            let parser = crate::parser::RbqParser::new(&crate::language::RbqLanguage {});
            let mut cache = oak_core::parser::ParseSession::default();
            let output = parser.parse(&source, &[], &mut cache);
            let root_green = match output.result {
                Ok(g) => g,
                Err(_) => return vec![],
            };
            let root = RedNode { green: root_green, offset: 0 };
            self.folding_provider.folding_ranges(&root).into_iter().map(|f| oak_lsp::FoldingRange { range: f.range, kind: f.kind }).collect()
        }
    }
    fn semantic_tokens<'a>(&'a self, uri: &'a str) -> impl Future<Output = Option<oak_lsp::SemanticTokens>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let source = self.vfs().request_source(&uri)?;
            let line_map = self.vfs().line_map(&uri)?;
            let parser = crate::parser::RbqParser::new(&crate::language::RbqLanguage {});
            let mut cache = oak_core::parser::ParseSession::default();
            let output = parser.parse(&source, &[], &mut cache);
            let root_green = output.result.ok()?;
            let root = RedNode { green: root_green, offset: 0 };
            use oak_semantic_tokens::SemanticTokensProvider;
            let tokens = self.semantic_tokens_provider.semantic_tokens(&root, &source, &line_map);
            Some(oak_lsp::SemanticTokens {
                result_id: None,
                data: tokens.into_iter().map(|t| oak_lsp::types::SemanticToken { delta_line: t.delta_line, delta_start: t.delta_start, length: t.length, token_type: t.token_type, token_modifiers_bitset: t.token_modifiers_bitset }).collect(),
            })
        }
    }
}
