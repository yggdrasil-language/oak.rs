//! `Lsp` 合同：把已建模的协议消息分派给 Oak 语言能力。
//!
//! 本模块是协议方法与语言能力之间的唯一分派点。传输适配者（stdio、HTTP、WebSocket）
//! 只负责 framing 与 JSON-RPC 编解码，把已解析的消息交给 [`Lsp::handle_request`] 与
//! [`Lsp::handle_notification`]，不自行 match 方法名，也不重复实现分派逻辑。

use crate::{
    protocol::{InitializeResult, Notification, Request, ResponseResult, ServerCapabilities},
    service::LanguageService,
};
use core::future::Future;

/// 语言服务器合同。
///
/// 任何 [`LanguageService`] 都通过 blanket impl 自动满足本 trait，因此现有语言实现
/// 无需改动就能接入任意传输适配者。传输适配者只依赖本 trait，不依赖具体语言类型。
pub trait Lsp: Send + Sync {
    /// 返回服务对外声明的能力集。
    fn capabilities(&self) -> ServerCapabilities;

    /// 处理一个已建模的请求并返回对应载荷。
    ///
    /// 参数按借用传入，请求中的 URI 与语言能力的方法签名共享同一生命周期，
    /// 因此分派过程不复制文档文本。
    fn handle_request<'a>(&'a self, request: &'a Request) -> impl Future<Output = ResponseResult> + Send + 'a;

    /// 处理一个已建模的通知。
    fn handle_notification<'a>(&'a self, notification: &'a Notification) -> impl Future<Output = ()> + Send + 'a;
}

impl<T: LanguageService> Lsp for T {
    fn capabilities(&self) -> ServerCapabilities {
        LanguageService::capabilities(self)
    }

    fn handle_request<'a>(&'a self, request: &'a Request) -> impl Future<Output = ResponseResult> + Send + 'a {
        async move {
            match request {
                Request::Initialize(params) => {
                    LanguageService::initialize(self, params.clone()).await;
                    ResponseResult::Initialize(InitializeResult { capabilities: LanguageService::capabilities(self) })
                }
                Request::Shutdown => {
                    LanguageService::shutdown(self).await;
                    ResponseResult::Shutdown
                }
                Request::Hover(request) => ResponseResult::Hover(LanguageService::hover(self, &request.uri, request.range).await),
                Request::Definition(request) => ResponseResult::Locations(LanguageService::definition(self, &request.uri, request.range).await),
                Request::References(request) => ResponseResult::Locations(LanguageService::references(self, &request.uri, request.range).await),
                Request::DocumentSymbol(request) => ResponseResult::DocumentSymbols(LanguageService::document_symbols(self, &request.uri).await),
                Request::FoldingRange(request) => ResponseResult::FoldingRanges(LanguageService::folding_ranges(self, &request.uri).await),
                Request::SemanticTokensFull(request) => ResponseResult::SemanticTokens(LanguageService::semantic_tokens(self, &request.uri).await),
                Request::Completion(request) => ResponseResult::Completion(LanguageService::completion(self, &request.uri, request.position).await),
                Request::DocumentHighlight(request) => ResponseResult::DocumentHighlights(LanguageService::document_highlight(self, &request.uri, request.range).await),
                Request::SignatureHelp(request) => ResponseResult::SignatureHelp(LanguageService::signature_help(self, &request.uri, request.range).await),
                Request::InlayHint(request) => ResponseResult::InlayHints(LanguageService::inlay_hint(self, &request.uri, request.range).await),
                Request::CodeAction(request) => ResponseResult::CodeActions(LanguageService::code_action(self, &request.uri, request.range).await),
                Request::Formatting(request) => ResponseResult::Edits(LanguageService::formatting(self, &request.uri).await),
                Request::RangeFormatting(request) => ResponseResult::Edits(LanguageService::range_formatting(self, &request.uri, request.range).await),
                Request::Rename(request) => ResponseResult::Rename(LanguageService::rename(self, &request.uri, request.range, request.new_name.clone()).await),
                Request::TypeDefinition(request) => ResponseResult::Locations(LanguageService::type_definition(self, &request.uri, request.range).await),
                Request::Implementation(request) => ResponseResult::Locations(LanguageService::implementation(self, &request.uri, request.range).await),
                Request::WorkspaceSymbol(request) => ResponseResult::WorkspaceSymbols(LanguageService::workspace_symbols(self, request.query.clone()).await),
                Request::Diagnostic(request) => ResponseResult::Diagnostics(LanguageService::diagnostics(self, &request.uri).await),
            }
        }
    }

    fn handle_notification<'a>(&'a self, notification: &'a Notification) -> impl Future<Output = ()> + Send + 'a {
        async move {
            match notification {
                Notification::Initialized => LanguageService::initialized(self).await,
                Notification::Exit => {}
                Notification::DidOpen(params) => LanguageService::did_open(self, &params.uri, &params.text).await,
                Notification::DidChange(params) => LanguageService::did_change(self, &params.uri, &params.text).await,
                Notification::DidClose(params) => LanguageService::did_close(self, &params.uri).await,
                Notification::DidSave(params) => LanguageService::did_save(self, &params.uri).await,
                Notification::WorkspaceDidChangeConfiguration => {}
            }
        }
    }
}
