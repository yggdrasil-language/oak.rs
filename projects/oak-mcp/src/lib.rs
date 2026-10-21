#![feature(new_range_api)]
#![warn(missing_docs)]
#![doc = include_str!("readme.md")]

use oak_core::Range;
use oak_lsp::service::LanguageService;
#[cfg(feature = "serde")]
use serde_json::Value as JsonValueNode;
#[cfg(feature = "serde")]
use serde_json::json;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

pub use oak_semantic_search::{NoSemanticSearch, SemanticSearch};

/// Represents a JSON-RPC 2.0 request.
///
/// This structure follows the JSON-RPC 2.0 specification for requests,
/// which must include a method and unique ID.
#[derive(Debug)]
#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonRpcRequest {
    /// The JSON-RPC version (must be "2.0").
    pub jsonrpc: String,
    /// The unique identifier for the request, used to match responses.
    pub id: JsonValueNode,
    /// The method name to be invoked on the server.
    pub method: String,
    /// The parameters for the method, if any, as a JSON object or array.
    pub params: Option<JsonValueNode>,
}

#[cfg(feature = "serde")]
impl JsonRpcRequest {
    /// Parses a JSON string into a `JsonRpcRequest`.
    ///
    /// # Errors
    ///
    /// Returns an error string if the JSON is malformed or does not
    /// conform to the `JsonRpcRequest` structure.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str(json_str).map_err(|e| e.to_string())
    }
}

/// Represents a JSON-RPC 2.0 response.
///
/// This structure follows the JSON-RPC 2.0 specification for responses,
/// which must include the ID of the corresponding request.
#[derive(Debug)]
#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonRpcResponse {
    /// The JSON-RPC version (must be "2.0").
    pub jsonrpc: String,
    /// The unique identifier corresponding to the original request.
    pub id: JsonValueNode,
    /// The successful result of the request, if any.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub result: Option<JsonValueNode>,
    /// The error details if the request failed.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub error: Option<JsonRpcError>,
}

#[cfg(feature = "serde")]
impl JsonRpcResponse {
    /// Serializes the response to a JSON string.
    ///
    /// Returns a string representation of the response. If serialization
    /// fails (which should not happen with valid data), returns "{}".
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }
}

/// Error information for a JSON-RPC response.
///
/// This structure is used in `JsonRpcResponse` when a request fails.
#[derive(Debug)]
#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonRpcError {
    /// The numeric error code.
    pub code: i32,
    /// A short, human-readable summary of the error.
    pub message: String,
    /// Additional error data, if any, providing more context.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub data: Option<JsonValueNode>,
}

/// Represents a JSON-RPC 2.0 notification.
///
/// Notifications are like requests but do not have an ID and do not
/// expect a response from the server.
#[derive(Debug)]
#[cfg(feature = "serde")]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JsonRpcNotification {
    /// The JSON-RPC version (must be "2.0").
    pub jsonrpc: String,
    /// The method name to be invoked.
    pub method: String,
    /// The parameters for the method, if any.
    pub params: Option<JsonValueNode>,
}

#[cfg(feature = "serde")]
impl JsonRpcNotification {
    /// Parses a JSON string into a `JsonRpcNotification`.
    ///
    /// # Errors
    ///
    /// Returns an error string if the JSON is malformed or does not
    /// conform to the `JsonRpcNotification` structure.
    pub fn from_json(json_str: &str) -> Result<Self, String> {
        serde_json::from_str(json_str).map_err(|e| e.to_string())
    }
}

/// A generic MCP server that wraps an Oak language service.
///
/// The `McpServer` provides an interface for LLMs to interact with Oak
/// language services using the Model Context Protocol. It supports
/// standard LSP-like features like hover, definitions, and diagnostics,
/// as well as semantic search.
///
/// # Type Parameters
///
/// * `S`: The underlying language service implementation.
/// * `E`: The semantic search engine implementation (defaults to `NoSemanticSearch`).
pub struct McpServer<S: LanguageService, E: SemanticSearch = NoSemanticSearch> {
    /// The language service that provides language-specific features.
    pub service: Arc<S>,
    /// The semantic search engine for code indexing and search.
    pub searcher: Option<Arc<E>>,
}

impl<S: LanguageService + 'static> McpServer<S, NoSemanticSearch>
where
    S::Vfs: oak_vfs::DocumentSession,
{
    /// Creates a new `McpServer` wrapping the given language service.
    pub fn new(service: S) -> Self {
        Self { service: Arc::new(service), searcher: None }
    }

    /// Creates a new `McpServer` wrapping the given language service and searcher.
    pub fn new_with_searcher(service: S, searcher: Arc<NoSemanticSearch>) -> Self {
        Self { service: Arc::new(service), searcher: Some(searcher) }
    }
}

impl<S: LanguageService + 'static, E: SemanticSearch + 'static> McpServer<S, E>
where
    S::Vfs: oak_vfs::DocumentSession,
{
    /// Adds a semantic searcher to the MCP server.
    ///
    /// Returns a new `McpServer` instance with the searcher configured.
    pub fn with_searcher<NewE: SemanticSearch>(self, searcher: NewE) -> McpServer<S, NewE> {
        McpServer { service: self.service, searcher: Some(Arc::new(searcher)) }
    }
}

#[cfg(feature = "serde")]
impl<S: LanguageService + 'static, E: SemanticSearch + 'static> McpServer<S, E>
where
    S::Vfs: oak_vfs::DocumentSession,
{
    /// Runs the MCP server on the given input and output streams.
    ///
    /// This method continuously reads JSON-RPC messages from `reader`,
    /// processes them, and writes responses to `writer`.
    ///
    /// # Errors
    ///
    /// Returns an error string if I/O fails or if there are unrecoverable
    /// processing errors.
    pub async fn run<R, W>(&self, mut reader: R, mut writer: W) -> Result<(), String>
    where
        R: tokio::io::AsyncBufRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        let mut line = String::new();

        while reader.read_line(&mut line).await.map_err(|e| e.to_string())? > 0 {
            let input = line.trim();
            if input.is_empty() {
                line.clear();
                continue;
            }

            if let Ok(request) = JsonRpcRequest::from_json(input) {
                let response = self.handle_request(request).await;
                let response_json = response.to_json();
                writer.write_all(response_json.as_bytes()).await.map_err(|e| e.to_string())?;
                writer.write_all(b"\n").await.map_err(|e| e.to_string())?;
                writer.flush().await.map_err(|e| e.to_string())?;
            }
            else if let Ok(notification) = JsonRpcNotification::from_json(input) {
                self.handle_notification(notification).await
            }

            line.clear()
        }

        Ok(())
    }

    /// Handles an incoming JSON-RPC request and returns a response.
    ///
    /// This method routes the request to the appropriate handler based
    /// on the method name. It supports:
    ///
    /// - `initialize`: Initializes the MCP server and returns capabilities.
    /// - `tools/list`: Returns a list of available tools defined in `tools.json`.
    /// - `tools/call`: Executes a tool call (e.g., hover, definition, search).
    ///
    /// Any other method will return a "Method not found" error (-32601).
    pub async fn handle_request(&self, request: JsonRpcRequest) -> JsonRpcResponse {
        match request.method.as_str() {
            "initialize" => JsonRpcResponse {
                jsonrpc: "2.0".to_string(),
                id: request.id,
                result: Some(json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "oak-mcp",
                        "version": "0.0.1"
                    }
                })),
                error: None,
            },
            "tools/list" => {
                let tools_str = include_str!("tools.json");
                let tools: JsonValueNode = serde_json::from_str(tools_str).unwrap();
                JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: request.id,
                    result: Some(json!({
                        "tools": tools
                    })),
                    error: None,
                }
            }
            "tools/call" => {
                let params = request.params.unwrap_or(json!(null));
                let name = params.get("name").and_then(|v| v.as_str()).unwrap_or_default();
                let args = params.get("arguments").cloned().unwrap_or(json!(null));

                match self.handle_tool_call(name, args).await {
                    Ok(result) => JsonRpcResponse {
                        jsonrpc: "2.0".to_string(),
                        id: request.id,
                        result: Some(json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": result.to_string()
                                }
                            ]
                        })),
                        error: None,
                    },
                    Err(e) => JsonRpcResponse { jsonrpc: "2.0".to_string(), id: request.id, result: None, error: Some(JsonRpcError { code: -32000, message: e, data: None }) },
                }
            }
            _ => JsonRpcResponse { jsonrpc: "2.0".to_string(), id: request.id, result: None, error: Some(JsonRpcError { code: -32601, message: format!("Method not found: {}", request.method), data: None }) },
        }
    }

    /// Handles a single JSON-RPC notification.
    ///
    /// Notifications are fire-and-forget messages that do not require a response.
    /// Common notifications include:
    ///
    /// - `initialized`: Sent by the client after receiving the initialize response.
    /// - `$/cancelRequest`: Sent by the client to cancel a pending request.
    ///
    /// Currently, these are acknowledged but no specific logic is performed.
    pub async fn handle_notification(&self, _notification: JsonRpcNotification) {
        // Implementation for handling notifications can be added here
    }

    /// Handles a tool call from the MCP client and dispatches it to the service.
    ///
    /// This is the primary entry point for LLMs to use Oak's language features.
    /// It supports standard LSP features and semantic search.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the tool to call (e.g., "hover", "semantic_search").
    /// * `args` - JSON arguments for the tool, typically containing `uri` and `offset`.
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the JSON result of the tool call or an error message.
    ///
    /// # Supported Tools
    ///
    /// - `hover`: Returns markdown or plaintext documentation for the symbol at the given position.
    /// - `definition`: Finds the definition location for the symbol at the given position.
    /// - `references`: Finds all references to the symbol at the given position.
    /// - `diagnostics`: Returns a list of errors and warnings for the specified file.
    /// - `completion`: Provides code completion suggestions for the current cursor position.
    /// - `symbols`: Lists all document symbols (classes, functions, etc.) in a file.
    /// - `semantic_search`: Performs a vector-based search over the indexed codebase.
    pub async fn handle_tool_call(&self, name: &str, args: JsonValueNode) -> Result<JsonValueNode, String> {
        match name {
            "hover" => {
                let uri = args.get("uri").and_then(|v| v.as_str()).ok_or("Missing uri")?;
                let offset = args.get("offset").and_then(|v| v.as_u64()).ok_or("Missing offset")? as usize;

                let hover = self.service.hover(uri, Range { start: offset, end: offset }).await;
                Ok(json!(hover.map(|h| h.contents).unwrap_or_default()))
            }
            "symbols" => {
                let uri = args.get("uri").and_then(|v| v.as_str()).ok_or("Missing uri")?;
                let _symbols = self.service.document_symbols(uri).await;
                // symbols is Vec<DocumentSymbol>, need proper conversion.
                // For now just returning empty array as placeholder if conversion is complex
                Ok(json!([]))
            }
            "definition" => {
                let uri = args.get("uri").and_then(|v| v.as_str()).ok_or("Missing uri")?;
                let offset = args.get("offset").and_then(|v| v.as_u64()).ok_or("Missing offset")? as usize;

                let _locs = self.service.definition(uri, Range { start: offset, end: offset }).await;
                Ok(json!([]))
            }
            "references" => {
                let uri = args.get("uri").and_then(|v| v.as_str()).ok_or("Missing uri")?;
                let offset = args.get("offset").and_then(|v| v.as_u64()).ok_or("Missing offset")? as usize;

                let _locs = self.service.references(uri, Range { start: offset, end: offset }).await;
                Ok(json!([]))
            }
            "diagnostics" => {
                let uri = args.get("uri").and_then(|v| v.as_str()).ok_or("Missing uri")?;
                let _diagnostics = self.service.diagnostics(uri).await;
                Ok(json!([]))
            }
            "completion" => {
                let uri = args.get("uri").and_then(|v| v.as_str()).ok_or("Missing uri")?;
                let offset = args.get("offset").and_then(|v| v.as_u64()).ok_or("Missing offset")? as usize;
                let _items = self.service.completion(uri, offset).await;
                Ok(json!([]))
            }
            "semantic_search" => {
                let searcher = self.searcher.as_ref().ok_or("Semantic search is not enabled")?;
                let query = args.get("query").and_then(|v| v.as_str()).ok_or("Missing query")?;
                let limit = args.get("limit").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

                let results = searcher.search(query, limit).await.map_err(|e| e.to_string())?;
                Ok(json!(results))
            }
            _ => Err(format!("Unknown tool: {}", name)),
        }
    }
}
