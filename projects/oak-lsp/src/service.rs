use crate::{
    protocol::{InitializeResult, ServerCapabilities},
    types::{CodeAction, CompletionItem, Diagnostic, DocumentHighlight, FoldingRange, Hover, InitializeParams, InlayHint, LocationRange, SemanticTokens, SignatureHelp, StructureItem, TextEdit, WorkspaceEdit, WorkspaceSymbol},
};
use core::range::Range;
use oak_core::{
    language::{ElementRole, ElementType, Language},
    source::Source,
    tree::RedNode,
};
use oak_folding::FoldingProvider;
use oak_hover::HoverProvider;
use oak_navigation::{DefinitionProvider, ReferencesProvider};
use oak_resolver::ModuleResolver;
use oak_semantic_tokens::SemanticTokensProvider;
use oak_symbols::SymbolProvider;
use oak_core::source::LineMap;
use oak_vfs::{Vfs, WritableVfs};
use std::future::Future;

/// A trait that defines the capabilities and behavior of a language-specific service.
///
/// This trait is the primary interface for implementing Language Server Protocol (LSP)
/// features. It provides hooks for various IDE features like hover, completion,
/// diagnostics, and symbol navigation.
///
/// # Implementation
///
/// Implementors should provide language-specific logic for parsing, resolving symbols,
/// and generating IDE-specific data structures.
pub trait LanguageService: Send + Sync {
    /// The language type this service supports.
    type Lang: Language;
    /// The VFS type used for source management.
    type Vfs: WritableVfs;

    /// Returns a reference to the underlying Virtual File System.
    fn vfs(&self) -> &Self::Vfs;

    /// Returns a reference to the workspace manager.
    fn workspace(&self) -> &crate::workspace::WorkspaceManager;

    /// Retrieves the source content for a given URI from the VFS.
    fn get_source(&self, uri: &str) -> Option<<Self::Vfs as Vfs>::Source> {
        self.vfs().get_source(uri)
    }
    /// Executes a closure with the root red node of a file.
    ///
    /// This is a convenience helper for running logic that requires the syntax tree.
    fn with_root<R, F>(&self, uri: &str, f: F) -> impl Future<Output = Option<R>> + Send
    where
        R: Send,
        F: FnOnce(RedNode<'_, Self::Lang>) -> R + Send;

    /// 返回本服务的 hover 能力实现；默认没有。
    ///
    /// 能力语义由 `oak-hover` 拥有，服务只提供接缝，不重复实现内容生成。
    fn hover_provider(&self) -> Option<&(dyn HoverProvider<Self::Lang> + Send + Sync)> {
        None
    }

    /// 返回本服务的定义跳转能力实现；默认没有。
    fn definition_provider(&self) -> Option<&(dyn DefinitionProvider<Self::Lang> + Send + Sync)> {
        None
    }

    /// 返回本服务的引用查找能力实现；默认没有。
    fn references_provider(&self) -> Option<&(dyn ReferencesProvider<Self::Lang> + Send + Sync)> {
        None
    }

    /// 返回本服务的符号能力实现；默认没有。
    fn symbol_provider(&self) -> Option<&(dyn SymbolProvider<Self::Lang> + Send + Sync)> {
        None
    }

    /// 返回本服务的折叠能力实现；默认没有。
    fn folding_provider(&self) -> Option<&(dyn FoldingProvider<Self::Lang> + Send + Sync)> {
        None
    }

    /// 返回本服务的语义 token 能力实现；默认没有。
    fn semantic_tokens_provider(&self) -> Option<&(dyn SemanticTokensProvider<Self::Lang> + Send + Sync)> {
        None
    }
    /// Executes a closure with multiple root nodes.
    fn with_roots<'a, R, F>(&'a self, uris: Vec<String>, f: F) -> impl Future<Output = Vec<R>> + Send + 'a
    where
        R: Send + 'static,
        F: Fn(RedNode<'_, Self::Lang>) -> R + Send + Sync + 'a,
    {
        let f = std::sync::Arc::new(f);
        async move {
            let mut results = Vec::new();
            for uri in uris {
                let f_clone = f.clone();
                if let Some(r) = self.with_root(&uri, move |node| f_clone(node)).await {
                    results.push(r);
                }
            }
            results
        }
    }

    /// Provides hover information for a specific range in a file.
    ///
    /// # Arguments
    /// * `uri` - The URI of the file.
    /// * `range` - The byte range to provide information for.
    fn hover<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Option<Hover>> + Send + 'a {
        async move {
            let provider = self.hover_provider()?;
            self.with_root(uri, |root| provider.hover(&root, range)).await.flatten()
        }
    }

    /// Provides folding ranges for a file.
    fn folding_ranges<'a>(&'a self, uri: &'a str) -> impl Future<Output = Vec<FoldingRange>> + Send + 'a {
        async move {
            let Some(provider) = self.folding_provider()
            else {
                return vec![];
            };
            self.with_root(uri, |root| provider.folding_ranges(&root)).await.unwrap_or_default()
        }
    }

    /// Provides document symbols (structure) for a file.
    ///
    /// 优先委托 `oak-symbols` 的 provider；未提供 provider 时回退到工作区符号索引。
    fn document_symbols<'a>(&'a self, uri: &'a str) -> impl Future<Output = Vec<StructureItem>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            if let (Some(provider), Some(source)) = (self.symbol_provider(), self.get_source(&uri)) {
                let symbols = self.with_root(&uri, |root| provider.document_symbols(&uri, &root, &source)).await;
                if let Some(symbols) = symbols {
                    return symbols.into_iter().map(crate::types::structure_item_from_symbol).collect();
                }
            }
            let _ = self.with_root(&uri, |_| {}).await;
            let symbols = self.workspace().symbols.query_file(&uri);
            if !symbols.is_empty() {
                return symbols.into_iter().map(crate::types::structure_item_from_symbol).collect();
            }
            vec![]
        }
    }

    /// Provides workspace-wide symbol search based on a query string.
    ///
    /// This method searches across all files in the workspace for symbols
    /// that match the given query string.
    fn workspace_symbols<'a>(&'a self, query: String) -> impl Future<Output = Vec<WorkspaceSymbol>> + Send + 'a {
        async move { self.workspace().symbols.query(&query).into_iter().map(|s| WorkspaceSymbol::from(s)).collect() }
    }

    /// Recursively lists all files in the VFS starting from the given root URI.
    ///
    /// This is used to discover all relevant source files in a workspace or directory.
    fn list_all_files(&self, root_uri: &str) -> impl Future<Output = Vec<String>> + Send + '_ {
        let root_uri: oak_core::Arc<str> = root_uri.into();
        async move {
            let mut files = Vec::new();
            let mut stack = vec![root_uri];

            while let Some(uri) = stack.pop() {
                if self.vfs().is_file(&uri) {
                    files.push(uri.to_string());
                }
                else if self.vfs().is_dir(&uri) {
                    if let Some(entries) = self.vfs().read_dir(&uri) {
                        stack.extend(entries);
                    }
                }
            }
            files
        }
    }

    /// Finds the definition(s) of a symbol at the specified range.
    ///
    /// 优先委托 `oak-navigation` 的 provider，其次使用工作区符号索引与模块解析。
    fn definition<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Vec<LocationRange>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            if let Some(provider) = self.definition_provider() {
                if let Some(found) = self.with_root(&uri, |root| provider.definition(&root, range.start)).await {
                    return found;
                }
            }
            let uri_clone = uri.clone();
            self.with_root(&uri, move |root| {
                let source = match self.get_source(&uri_clone) {
                    Some(s) => s,
                    None => return vec![],
                };
                use oak_core::tree::RedTree;
                let node = match root.child_at_offset(range.start) {
                    Some(RedTree::Node(n)) => n,
                    Some(RedTree::Leaf(l)) => return vec![LocationRange { uri: uri_clone.clone().into(), range: l.span }],
                    None => root,
                };
                let role = node.green.kind.role();
                if role.universal() == oak_core::language::UniversalElementRole::Reference {
                    let name = &source.get_text_in(node.span());
                    if let Some(sym) = self.workspace().symbols.lookup(name) {
                        return vec![LocationRange { uri: sym.uri, range: sym.range }];
                    }
                    if let Some(resolved_uri) = self.workspace().resolver.resolve(&uri_clone, name) {
                        return vec![LocationRange { uri: resolved_uri.into(), range: (0..0).into() }];
                    }
                }
                vec![]
            })
            .await
            .unwrap_or_default()
        }
    }

    /// Provides document highlights for a symbol at the specified range.
    fn document_highlight<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Vec<DocumentHighlight>> + Send + 'a {
        async { vec![] }
    }

    /// Provides code actions for a specific range in a file.
    fn code_action<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Vec<CodeAction>> + Send + 'a {
        async { vec![] }
    }

    /// Provides formatting edits for a file.
    fn formatting<'a>(&'a self, _uri: &'a str) -> impl Future<Output = Vec<TextEdit>> + Send + 'a {
        async { vec![] }
    }

    /// Provides range formatting edits for a file.
    fn range_formatting<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Vec<TextEdit>> + Send + 'a {
        async { vec![] }
    }

    /// Provides rename edits for a symbol at the specified range.
    fn rename<'a>(&'a self, _uri: &'a str, _range: Range<usize>, _new_name: String) -> impl Future<Output = Option<WorkspaceEdit>> + Send + 'a {
        async { None }
    }

    /// Provides semantic tokens for a file.
    fn semantic_tokens<'a>(&'a self, uri: &'a str) -> impl Future<Output = Option<SemanticTokens>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let provider = self.semantic_tokens_provider()?;
            let source = self.get_source(&uri)?;
            let line_map = LineMap::from_source(&source);
            let data = self.with_root(&uri, |root| provider.semantic_tokens(&root, &source, &line_map)).await?;
            Some(SemanticTokens { result_id: None, data })
        }
    }

    /// Provides completion items for a file at the specified position.
    ///
    /// Implementations should use the syntax tree and symbol index to suggest
    /// relevant keywords, identifiers, and snippets based on the cursor context.
    fn completion<'a>(&'a self, _uri: &'a str, _offset: usize) -> impl Future<Output = Vec<CompletionItem>> + Send + 'a {
        async { vec![] }
    }

    /// Provides signature help for a symbol at the specified range.
    ///
    /// Typically used for function or method calls to show parameter information
    /// and documentation as the user types.
    fn signature_help<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Option<SignatureHelp>> + Send + 'a {
        async { None }
    }

    /// Provides inlay hints for a file.
    ///
    /// Inlay hints are small pieces of information shown inline with the code,
    /// such as parameter names or inferred types.
    fn inlay_hint<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Vec<InlayHint>> + Send + 'a {
        async { vec![] }
    }

    /// Finds all references to a symbol at the specified range.
    fn references<'a>(&'a self, uri: &'a str, range: Range<usize>) -> impl Future<Output = Vec<LocationRange>> + Send + 'a {
        let uri = uri.to_string();
        async move {
            let Some(provider) = self.references_provider()
            else {
                return vec![];
            };
            self.with_root(&uri, |root| provider.references(&root, range.start, true)).await.unwrap_or_default()
        }
    }

    /// Finds the type definition of a symbol at the specified range.
    fn type_definition<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Vec<LocationRange>> + Send + 'a {
        async { vec![] }
    }

    /// Finds the implementation(s) of a symbol at the specified range.
    fn implementation<'a>(&'a self, _uri: &'a str, _range: Range<usize>) -> impl Future<Output = Vec<LocationRange>> + Send + 'a {
        async { vec![] }
    }

    /// 返回本服务对外声明的能力集。
    ///
    /// 默认由六个能力接缝推导；直接实现额外能力（如补全、诊断）的服务应覆盖本方法，
    /// 否则这些能力不会出现在 `initialize` 结果中。
    fn capabilities(&self) -> ServerCapabilities {
        ServerCapabilities {
            hover: self.hover_provider().is_some(),
            definition: self.definition_provider().is_some(),
            references: self.references_provider().is_some(),
            document_symbol: self.symbol_provider().is_some(),
            folding_range: self.folding_provider().is_some(),
            semantic_tokens: self.semantic_tokens_provider().is_some(),
            ..ServerCapabilities::default()
        }
    }

    /// 文档被打开：把完整文本写入 VFS。
    fn did_open<'a>(&'a self, uri: &'a str, text: &'a str) -> impl Future<Output = ()> + Send + 'a {
        async move {
            self.vfs().write_file(uri, text.into());
        }
    }

    /// 文档内容变化：把完整文本写入 VFS。
    ///
    /// Oak 定义层只建模全量同步，因此与 [`Self::did_open`] 共用写入路径。
    fn did_change<'a>(&'a self, uri: &'a str, text: &'a str) -> impl Future<Output = ()> + Send + 'a {
        async move {
            self.did_open(uri, text).await;
        }
    }

    /// Handles an LSP initialize request.
    fn initialize<'a>(&'a self, _params: InitializeParams) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }

    /// Called when the language server is fully initialized.
    fn initialized<'a>(&'a self) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }

    /// Called when the language server is shut down.
    fn shutdown<'a>(&'a self) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }

    /// Called when a file is saved in the editor.
    fn did_save<'a>(&'a self, _uri: &'a str) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }

    /// Called when a file is closed in the editor.
    fn did_close<'a>(&'a self, _uri: &'a str) -> impl Future<Output = ()> + Send + 'a {
        async {}
    }

    /// Provides diagnostics for a file.
    fn diagnostics<'a>(&'a self, _uri: &'a str) -> impl Future<Output = Vec<Diagnostic>> + Send + 'a {
        async { vec![] }
    }
}

/// LSP 会话生命周期合同。
///
/// 这是提供给传输/会话适配层的窄视图：适配层只需要初始化、关闭与能力集，
/// 不必依赖 [`LanguageService`] 的全部语言能力。任何 `LanguageService` 都通过
/// 下面的 blanket impl 自动获得本合同。
pub trait LifecycleService {
    /// 建立会话并返回协议要求的初始化结果。
    ///
    /// 实现的初始化钩子完成后，结果中的能力集来自 [`LanguageService::capabilities`]。
    fn initialize<'a>(&'a self, params: InitializeParams) -> impl Future<Output = InitializeResult> + Send + 'a;

    /// 客户端确认已收到初始化结果。
    fn initialized<'a>(&'a self) -> impl Future<Output = ()> + Send + 'a;

    /// 关闭服务。
    fn shutdown<'a>(&'a self) -> impl Future<Output = ()> + Send + 'a;

    /// 返回服务支持的能力集。
    fn capabilities(&self) -> ServerCapabilities;
}

impl<T: LanguageService> LifecycleService for T {
    fn initialize<'a>(&'a self, params: InitializeParams) -> impl Future<Output = InitializeResult> + Send + 'a {
        async move {
            LanguageService::initialize(self, params).await;
            InitializeResult { capabilities: LanguageService::capabilities(self) }
        }
    }

    fn initialized<'a>(&'a self) -> impl Future<Output = ()> + Send + 'a {
        LanguageService::initialized(self)
    }

    fn shutdown<'a>(&'a self) -> impl Future<Output = ()> + Send + 'a {
        LanguageService::shutdown(self)
    }

    fn capabilities(&self) -> ServerCapabilities {
        LanguageService::capabilities(self)
    }
}

/// 文档同步合同。
///
/// 与 [`LifecycleService`] 一样，这是 transport 适配层使用的窄视图；语言实现仍然在
/// [`LanguageService`] 上覆盖同样的方法，避免要求每个实现重复声明合同。
pub trait DocumentSync {
    /// 文档被打开。
    fn did_open<'a>(&'a self, uri: &'a str, text: &'a str) -> impl Future<Output = ()> + Send + 'a;

    /// 文档内容变化。
    fn did_change<'a>(&'a self, uri: &'a str, text: &'a str) -> impl Future<Output = ()> + Send + 'a;

    /// 文档被关闭。
    fn did_close<'a>(&'a self, uri: &'a str) -> impl Future<Output = ()> + Send + 'a;

    /// 文档被保存。
    fn did_save<'a>(&'a self, uri: &'a str) -> impl Future<Output = ()> + Send + 'a;
}

impl<T: LanguageService> DocumentSync for T {
    fn did_open<'a>(&'a self, uri: &'a str, text: &'a str) -> impl Future<Output = ()> + Send + 'a {
        LanguageService::did_open(self, uri, text)
    }

    fn did_change<'a>(&'a self, uri: &'a str, text: &'a str) -> impl Future<Output = ()> + Send + 'a {
        LanguageService::did_change(self, uri, text)
    }

    fn did_close<'a>(&'a self, uri: &'a str) -> impl Future<Output = ()> + Send + 'a {
        LanguageService::did_close(self, uri)
    }

    fn did_save<'a>(&'a self, uri: &'a str) -> impl Future<Output = ()> + Send + 'a {
        LanguageService::did_save(self, uri)
    }
}
