//! LSP 协议消息与方法建模。
//!
//! 本模块只拥有协议身份：方法名、请求与通知的参数形状、每类方法的结果载荷，
//! 以及由能力接缝推导的服务能力集。它不拥有语言语义，也不驱动传输、会话或路由；
//! server 提供者在自己的归属里把线上报文映射为这里的 `Request` / `Notification`，
//! 再驱动 [`crate::service::LanguageService`]。

use crate::{
    handlers::{PositionRequest, QueryRequest, RangeRequest, RenameRequest, TextDocumentParams, UriRequest},
    types::{CodeAction, CompletionItem, Diagnostic, DocumentHighlight, FoldingRange, Hover, InitializeParams, InlayHint, LocationRange, SemanticTokens, SignatureHelp, StructureItem, TextEdit, WorkspaceEdit, WorkspaceSymbol},
};

/// LSP 请求方法身份。
///
/// 每个变体对应一个已建模的线上方法名，见 [`RequestMethod::as_str`]。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestMethod {
    /// `initialize`：建立会话并交换能力集。
    Initialize,
    /// `shutdown`：请求服务停止接收新请求。
    Shutdown,
    /// `textDocument/hover`：查询位置处的悬停信息。
    Hover,
    /// `textDocument/definition`：跳转到定义。
    Definition,
    /// `textDocument/references`：查找引用。
    References,
    /// `textDocument/documentSymbol`：列出文档结构。
    DocumentSymbol,
    /// `textDocument/foldingRange`：列出折叠范围。
    FoldingRange,
    /// `textDocument/semanticTokens/full`：请求整份文档的语义 token。
    SemanticTokensFull,
    /// `textDocument/completion`：请求补全项。
    Completion,
    /// `textDocument/documentHighlight`：请求文档内高亮。
    DocumentHighlight,
    /// `textDocument/signatureHelp`：请求签名帮助。
    SignatureHelp,
    /// `textDocument/inlayHint`：请求行内提示。
    InlayHint,
    /// `textDocument/codeAction`：请求代码操作。
    CodeAction,
    /// `textDocument/formatting`：请求整份文档格式化。
    Formatting,
    /// `textDocument/rangeFormatting`：请求区间格式化。
    RangeFormatting,
    /// `textDocument/rename`：请求重命名编辑。
    Rename,
    /// `textDocument/typeDefinition`：跳转到类型定义。
    TypeDefinition,
    /// `textDocument/implementation`：跳转到实现。
    Implementation,
    /// `workspace/symbol`：工作区符号搜索。
    WorkspaceSymbol,
    /// `textDocument/diagnostic`：拉取文档诊断。
    Diagnostic,
}

impl RequestMethod {
    /// 返回线上方法名。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initialize => "initialize",
            Self::Shutdown => "shutdown",
            Self::Hover => "textDocument/hover",
            Self::Definition => "textDocument/definition",
            Self::References => "textDocument/references",
            Self::DocumentSymbol => "textDocument/documentSymbol",
            Self::FoldingRange => "textDocument/foldingRange",
            Self::SemanticTokensFull => "textDocument/semanticTokens/full",
            Self::Completion => "textDocument/completion",
            Self::DocumentHighlight => "textDocument/documentHighlight",
            Self::SignatureHelp => "textDocument/signatureHelp",
            Self::InlayHint => "textDocument/inlayHint",
            Self::CodeAction => "textDocument/codeAction",
            Self::Formatting => "textDocument/formatting",
            Self::RangeFormatting => "textDocument/rangeFormatting",
            Self::Rename => "textDocument/rename",
            Self::TypeDefinition => "textDocument/typeDefinition",
            Self::Implementation => "textDocument/implementation",
            Self::WorkspaceSymbol => "workspace/symbol",
            Self::Diagnostic => "textDocument/diagnostic",
        }
    }

    /// 解析线上方法名；未建模的方法返回 `None`。
    ///
    /// 返回 `None` 表示适配层无需为该请求建模，而不是请求非法。
    pub fn parse(method: &str) -> Option<Self> {
        Some(match method {
            "initialize" => Self::Initialize,
            "shutdown" => Self::Shutdown,
            "textDocument/hover" => Self::Hover,
            "textDocument/definition" => Self::Definition,
            "textDocument/references" => Self::References,
            "textDocument/documentSymbol" => Self::DocumentSymbol,
            "textDocument/foldingRange" => Self::FoldingRange,
            "textDocument/semanticTokens/full" => Self::SemanticTokensFull,
            "textDocument/completion" => Self::Completion,
            "textDocument/documentHighlight" => Self::DocumentHighlight,
            "textDocument/signatureHelp" => Self::SignatureHelp,
            "textDocument/inlayHint" => Self::InlayHint,
            "textDocument/codeAction" => Self::CodeAction,
            "textDocument/formatting" => Self::Formatting,
            "textDocument/rangeFormatting" => Self::RangeFormatting,
            "textDocument/rename" => Self::Rename,
            "textDocument/typeDefinition" => Self::TypeDefinition,
            "textDocument/implementation" => Self::Implementation,
            "workspace/symbol" => Self::WorkspaceSymbol,
            "textDocument/diagnostic" => Self::Diagnostic,
            _ => return None,
        })
    }
}

/// LSP 通知方法身份。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NotificationMethod {
    /// `initialized`：客户端确认已收到初始化结果。
    Initialized,
    /// `exit`：客户端请求服务进程退出。
    Exit,
    /// `textDocument/didOpen`：文档被打开。
    DidOpen,
    /// `textDocument/didChange`：文档内容变化。
    DidChange,
    /// `textDocument/didClose`：文档被关闭。
    DidClose,
    /// `textDocument/didSave`：文档被保存。
    DidSave,
    /// `workspace/didChangeConfiguration`：工作区配置变化。
    WorkspaceDidChangeConfiguration,
}

impl NotificationMethod {
    /// 返回线上方法名。
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Initialized => "initialized",
            Self::Exit => "exit",
            Self::DidOpen => "textDocument/didOpen",
            Self::DidChange => "textDocument/didChange",
            Self::DidClose => "textDocument/didClose",
            Self::DidSave => "textDocument/didSave",
            Self::WorkspaceDidChangeConfiguration => "workspace/didChangeConfiguration",
        }
    }

    /// 解析线上方法名；未建模的方法返回 `None`。
    pub fn parse(method: &str) -> Option<Self> {
        Some(match method {
            "initialized" => Self::Initialized,
            "exit" => Self::Exit,
            "textDocument/didOpen" => Self::DidOpen,
            "textDocument/didChange" => Self::DidChange,
            "textDocument/didClose" => Self::DidClose,
            "textDocument/didSave" => Self::DidSave,
            "workspace/didChangeConfiguration" => Self::WorkspaceDidChangeConfiguration,
            _ => return None,
        })
    }
}

/// 已建模的 LSP 请求，携带该方法对应的参数。
///
/// 参数沿用 Oak 的字节偏移索引：适配层负责把线上的行/列位置换算成字节偏移，
/// 定义层不做第二套源映射。
#[derive(Debug, Clone)]
pub enum Request {
    /// `initialize` 请求。
    Initialize(InitializeParams),
    /// `shutdown` 请求，无参数。
    Shutdown,
    /// `textDocument/hover` 请求。
    Hover(RangeRequest),
    /// `textDocument/definition` 请求。
    Definition(RangeRequest),
    /// `textDocument/references` 请求。
    References(RangeRequest),
    /// `textDocument/documentSymbol` 请求。
    DocumentSymbol(UriRequest),
    /// `textDocument/foldingRange` 请求。
    FoldingRange(UriRequest),
    /// `textDocument/semanticTokens/full` 请求。
    SemanticTokensFull(UriRequest),
    /// `textDocument/completion` 请求。
    Completion(PositionRequest),
    /// `textDocument/documentHighlight` 请求。
    DocumentHighlight(RangeRequest),
    /// `textDocument/signatureHelp` 请求。
    SignatureHelp(RangeRequest),
    /// `textDocument/inlayHint` 请求。
    InlayHint(RangeRequest),
    /// `textDocument/codeAction` 请求。
    CodeAction(RangeRequest),
    /// `textDocument/formatting` 请求。
    Formatting(UriRequest),
    /// `textDocument/rangeFormatting` 请求。
    RangeFormatting(RangeRequest),
    /// `textDocument/rename` 请求。
    Rename(RenameRequest),
    /// `textDocument/typeDefinition` 请求。
    TypeDefinition(RangeRequest),
    /// `textDocument/implementation` 请求。
    Implementation(RangeRequest),
    /// `workspace/symbol` 请求。
    WorkspaceSymbol(QueryRequest),
    /// `textDocument/diagnostic` 请求。
    Diagnostic(UriRequest),
}

impl Request {
    /// 返回该请求的方法身份。
    pub const fn method(&self) -> RequestMethod {
        match self {
            Self::Initialize(_) => RequestMethod::Initialize,
            Self::Shutdown => RequestMethod::Shutdown,
            Self::Hover(_) => RequestMethod::Hover,
            Self::Definition(_) => RequestMethod::Definition,
            Self::References(_) => RequestMethod::References,
            Self::DocumentSymbol(_) => RequestMethod::DocumentSymbol,
            Self::FoldingRange(_) => RequestMethod::FoldingRange,
            Self::SemanticTokensFull(_) => RequestMethod::SemanticTokensFull,
            Self::Completion(_) => RequestMethod::Completion,
            Self::DocumentHighlight(_) => RequestMethod::DocumentHighlight,
            Self::SignatureHelp(_) => RequestMethod::SignatureHelp,
            Self::InlayHint(_) => RequestMethod::InlayHint,
            Self::CodeAction(_) => RequestMethod::CodeAction,
            Self::Formatting(_) => RequestMethod::Formatting,
            Self::RangeFormatting(_) => RequestMethod::RangeFormatting,
            Self::Rename(_) => RequestMethod::Rename,
            Self::TypeDefinition(_) => RequestMethod::TypeDefinition,
            Self::Implementation(_) => RequestMethod::Implementation,
            Self::WorkspaceSymbol(_) => RequestMethod::WorkspaceSymbol,
            Self::Diagnostic(_) => RequestMethod::Diagnostic,
        }
    }

    /// 返回该请求的线上方法名。
    pub const fn method_str(&self) -> &'static str {
        self.method().as_str()
    }
}

/// 已建模的 LSP 通知。
#[derive(Debug, Clone)]
pub enum Notification {
    /// `initialized` 通知，无参数。
    Initialized,
    /// `exit` 通知，无参数。
    Exit,
    /// `textDocument/didOpen` 通知。
    DidOpen(TextDocumentParams),
    /// `textDocument/didChange` 通知。
    DidChange(TextDocumentParams),
    /// `textDocument/didClose` 通知。
    DidClose(UriRequest),
    /// `textDocument/didSave` 通知。
    DidSave(UriRequest),
    /// `workspace/didChangeConfiguration` 通知，无参数。
    WorkspaceDidChangeConfiguration,
}

impl Notification {
    /// 返回该通知的方法身份。
    pub const fn method(&self) -> NotificationMethod {
        match self {
            Self::Initialized => NotificationMethod::Initialized,
            Self::Exit => NotificationMethod::Exit,
            Self::DidOpen(_) => NotificationMethod::DidOpen,
            Self::DidChange(_) => NotificationMethod::DidChange,
            Self::DidClose(_) => NotificationMethod::DidClose,
            Self::DidSave(_) => NotificationMethod::DidSave,
            Self::WorkspaceDidChangeConfiguration => NotificationMethod::WorkspaceDidChangeConfiguration,
        }
    }

    /// 返回该通知的线上方法名。
    pub const fn method_str(&self) -> &'static str {
        self.method().as_str()
    }
}

/// `initialize` 请求的结果。
#[derive(Debug, Clone)]
pub struct InitializeResult {
    /// 服务支持的能力集。
    pub capabilities: ServerCapabilities,
}

/// 已建模方法的响应结果载荷。
///
/// 结果按能力族归并：同一族的方法共享同一种载荷形状，调用方按 [`Request::method`]
/// 决定如何解释结果，而不是依赖方法字符串。
#[derive(Debug, Clone)]
pub enum ResponseResult {
    /// `initialize` 的结果。
    Initialize(InitializeResult),
    /// `shutdown` 的结果，无载荷。
    Shutdown,
    /// `textDocument/hover` 的结果。
    Hover(Option<Hover>),
    /// `textDocument/definition`、`references`、`typeDefinition`、`implementation` 的结果。
    Locations(Vec<LocationRange>),
    /// `textDocument/documentSymbol` 的结果。
    DocumentSymbols(Vec<StructureItem>),
    /// `textDocument/foldingRange` 的结果。
    FoldingRanges(Vec<FoldingRange>),
    /// `textDocument/semanticTokens/full` 的结果。
    SemanticTokens(Option<SemanticTokens>),
    /// `textDocument/completion` 的结果。
    Completion(Vec<CompletionItem>),
    /// `textDocument/documentHighlight` 的结果。
    DocumentHighlights(Vec<DocumentHighlight>),
    /// `textDocument/signatureHelp` 的结果。
    SignatureHelp(Option<SignatureHelp>),
    /// `textDocument/inlayHint` 的结果。
    InlayHints(Vec<InlayHint>),
    /// `textDocument/codeAction` 的结果。
    CodeActions(Vec<CodeAction>),
    /// `textDocument/formatting` 与 `rangeFormatting` 的结果。
    Edits(Vec<TextEdit>),
    /// `textDocument/rename` 的结果。
    Rename(Option<WorkspaceEdit>),
    /// `workspace/symbol` 的结果。
    WorkspaceSymbols(Vec<WorkspaceSymbol>),
    /// `textDocument/diagnostic` 的结果。
    Diagnostics(Vec<Diagnostic>),
}

/// 文档同步方式。
///
/// Oak 定义层只建模全量同步：`didOpen` / `didChange` 携带完整文本，
/// 服务写入自己的 VFS。增量同步需要第二套源映射，不属于定义层。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum TextDocumentSyncKind {
    /// 全量同步。
    Full = 1,
}

/// 服务能力集。
///
/// 能力由 [`crate::service::LanguageService`] 的能力接缝推导，见
/// [`crate::service::LifecycleService::capabilities`]；未建模为能力接缝的方法保持不支持。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ServerCapabilities {
    /// 是否支持 `textDocument/hover`。
    pub hover: bool,
    /// 是否支持 `textDocument/definition`。
    pub definition: bool,
    /// 是否支持 `textDocument/references`。
    pub references: bool,
    /// 是否支持 `textDocument/documentSymbol`。
    pub document_symbol: bool,
    /// 是否支持 `textDocument/foldingRange`。
    pub folding_range: bool,
    /// 是否支持 `textDocument/semanticTokens/full`。
    pub semantic_tokens: bool,
    /// 是否支持 `textDocument/completion`。
    pub completion: bool,
    /// 是否支持 `textDocument/documentHighlight`。
    pub document_highlight: bool,
    /// 是否支持 `textDocument/signatureHelp`。
    pub signature_help: bool,
    /// 是否支持 `textDocument/inlayHint`。
    pub inlay_hint: bool,
    /// 是否支持 `textDocument/codeAction`。
    pub code_action: bool,
    /// 是否支持 `textDocument/formatting`。
    pub formatting: bool,
    /// 是否支持 `textDocument/rangeFormatting`。
    pub range_formatting: bool,
    /// 是否支持 `textDocument/rename`。
    pub rename: bool,
    /// 是否支持 `textDocument/typeDefinition`。
    pub type_definition: bool,
    /// 是否支持 `textDocument/implementation`。
    pub implementation: bool,
    /// 是否支持 `workspace/symbol`。
    pub workspace_symbol: bool,
    /// 是否支持 `textDocument/diagnostic`。
    pub diagnostic: bool,
    /// 文档同步方式。
    pub text_document_sync: TextDocumentSyncKind,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            hover: false,
            definition: false,
            references: false,
            document_symbol: false,
            folding_range: false,
            semantic_tokens: false,
            completion: false,
            document_highlight: false,
            signature_help: false,
            inlay_hint: false,
            code_action: false,
            formatting: false,
            range_formatting: false,
            rename: false,
            type_definition: false,
            implementation: false,
            workspace_symbol: false,
            diagnostic: false,
            text_document_sync: TextDocumentSyncKind::Full,
        }
    }
}

impl ServerCapabilities {
    /// 判断能力集是否支持某个请求方法。
    pub const fn supports(&self, method: RequestMethod) -> bool {
        match method {
            RequestMethod::Initialize | RequestMethod::Shutdown => true,
            RequestMethod::Hover => self.hover,
            RequestMethod::Definition => self.definition,
            RequestMethod::References => self.references,
            RequestMethod::DocumentSymbol => self.document_symbol,
            RequestMethod::FoldingRange => self.folding_range,
            RequestMethod::SemanticTokensFull => self.semantic_tokens,
            RequestMethod::Completion => self.completion,
            RequestMethod::DocumentHighlight => self.document_highlight,
            RequestMethod::SignatureHelp => self.signature_help,
            RequestMethod::InlayHint => self.inlay_hint,
            RequestMethod::CodeAction => self.code_action,
            RequestMethod::Formatting => self.formatting,
            RequestMethod::RangeFormatting => self.range_formatting,
            RequestMethod::Rename => self.rename,
            RequestMethod::TypeDefinition => self.type_definition,
            RequestMethod::Implementation => self.implementation,
            RequestMethod::WorkspaceSymbol => self.workspace_symbol,
            RequestMethod::Diagnostic => self.diagnostic,
        }
    }
}
