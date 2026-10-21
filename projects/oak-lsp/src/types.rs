use oak_core::{Arc, Range, language::UniversalElementRole};
pub use oak_folding::FoldingRange;
pub use oak_hover::Hover;
pub use oak_navigation::Location as LocationRange;
pub use oak_semantic_tokens::SemanticToken;
pub use oak_structural_view::StructureItem;

/// A range in the source code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LspRange {
    /// The start offset.
    pub start: usize,
    /// The end offset.
    pub end: usize,
}

impl From<Range<usize>> for LspRange {
    fn from(range: Range<usize>) -> Self {
        Self { start: range.start, end: range.end }
    }
}

impl From<LspRange> for Range<usize> {
    fn from(range: LspRange) -> Self {
        Self { start: range.start, end: range.end }
    }
}

/// A location in a resource.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound(serialize = "R: serde::Serialize", deserialize = "R: serde::Deserialize<'de>")))]
pub struct Location<R = Range<usize>> {
    /// The URI of the resource.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_arc_str"))]
    pub uri: Arc<str>,
    /// The range within the resource.
    pub range: R,
}

/// A specialized location type for byte-based ranges.
impl From<LocationRange> for Location<Range<usize>> {
    fn from(loc: LocationRange) -> Self {
        Self { uri: loc.uri, range: loc.range }
    }
}

impl From<Location<Range<usize>>> for LocationRange {
    fn from(loc: Location<Range<usize>>) -> Self {
        Self { uri: loc.uri, range: loc.range }
    }
}

/// A source position with line, column, and offset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourcePosition {
    /// Line number (1-indexed).
    pub line: u32,
    /// Column number (1-indexed).
    pub column: u32,
    /// Byte offset (0-indexed).
    pub offset: usize,
    /// Length of the token/element.
    pub length: usize,
}

/// Represents a source location with an optional URL.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceLocation {
    /// Line number (1-indexed).
    pub line: u32,
    /// Column number (1-indexed).
    pub column: u32,
    /// Optional URL of the source file.
    pub url: Option<url::Url>,
}

impl Default for SourceLocation {
    /// Creates a default `SourceLocation` at line 1, column 1.
    fn default() -> Self {
        Self { line: 1, column: 1, url: None }
    }
}

/// Represents a document highlight kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DocumentHighlightKind {
    /// A textual occurrence.
    Text = 1,
    /// Read-access of a symbol, like reading a variable.
    Read = 2,
    /// Write-access of a symbol, like writing to a variable.
    Write = 3,
}

/// Represents a document highlight.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DocumentHighlight {
    /// The range this highlight applies to.
    pub range: LspRange,
    /// The highlight kind, default is DocumentHighlightKind.Text.
    pub kind: Option<DocumentHighlightKind>,
}

/// Represents a color in RGBA space.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Color {
    /// The red component of this color in the range [0-1].
    pub red: f32,
    /// The green component of this color in the range [0-1].
    pub green: f32,
    /// The blue component of this color in the range [0-1].
    pub blue: f32,
    /// The alpha component of this color in the range [0-1].
    pub alpha: f32,
}

/// Represents a color range from a document.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ColorInformation {
    /// The range in the document where this color appears.
    pub range: LspRange,
    /// The actual color value for this range.
    pub color: Color,
}

/// Parameters for the `initialize` request.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InitializeParams {
    /// The root URI of the workspace.
    pub root_uri: Option<String>,
    /// The workspace folders.
    pub workspace_folders: Vec<WorkspaceFolder>,
}

/// A workspace folder.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkspaceFolder {
    /// The URI of the workspace folder.
    pub uri: String,
    /// The name of the workspace folder.
    pub name: String,
}

/// Represents a symbol kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SymbolKind {
    /// File symbol kind.
    File = 1,
    /// Module symbol kind.
    Module = 2,
    /// Namespace symbol kind.
    Namespace = 3,
    /// Package symbol kind.
    Package = 4,
    /// Class symbol kind.
    Class = 5,
    /// Method symbol kind.
    Method = 6,
    /// Property symbol kind.
    Property = 7,
    /// Field symbol kind.
    Field = 8,
    /// Constructor symbol kind.
    Constructor = 9,
    /// Enum symbol kind.
    Enum = 10,
    /// Interface symbol kind.
    Interface = 11,
    /// Function symbol kind.
    Function = 12,
    /// Variable symbol kind.
    Variable = 13,
    /// Constant symbol kind.
    Constant = 14,
    /// String symbol kind.
    String = 15,
    /// Number symbol kind.
    Number = 16,
    /// Boolean symbol kind.
    Boolean = 17,
    /// Array symbol kind.
    Array = 18,
    /// Object symbol kind.
    Object = 19,
    /// Key symbol kind.
    Key = 20,
    /// Null symbol kind.
    Null = 21,
    /// EnumMember symbol kind.
    EnumMember = 22,
    /// Struct symbol kind.
    Struct = 23,
    /// Event symbol kind.
    Event = 24,
    /// Operator symbol kind.
    Operator = 25,
    /// TypeParameter symbol kind.
    TypeParameter = 26,
}

/// Represents a workspace symbol.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkspaceSymbol {
    /// The name of the symbol.
    pub name: String,
    /// The kind of the symbol.
    pub kind: SymbolKind,
    /// The location of the symbol.
    pub location: LocationRange,
    /// The name of the container this symbol is in.
    pub container_name: Option<String>,
}

/// Represents information about a symbol (e.g., function, variable, class).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SymbolInformation {
    /// The name of the symbol.
    pub name: String,
    /// The kind of the symbol.
    pub kind: SymbolKind,
    /// The location of the symbol.
    pub location: LocationRange,
    /// The name of the container this symbol is in.
    pub container_name: Option<String>,
}

/// Represents a change to the workspace.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WorkspaceEdit {
    /// The changes to the workspace.
    pub changes: std::collections::HashMap<String, Vec<TextEdit>>,
}

/// Represents a text edit.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextEdit {
    /// The range of the text edit.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub range: Range<usize>,
    /// The new text.
    pub new_text: String,
}

/// Represents a completion item.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CompletionItem {
    /// The label of the completion item.
    pub label: String,
    /// The kind of the completion item.
    pub kind: Option<CompletionItemKind>,
    /// A human-readable string with additional information about this item.
    pub detail: Option<String>,
    /// A human-readable string that contains documentation about this item.
    pub documentation: Option<String>,
    /// The text that should be inserted when selecting this completion item.
    pub insert_text: Option<String>,
}

/// Represents a completion item kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum CompletionItemKind {
    /// Text completion.
    Text = 1,
    /// Method completion.
    Method = 2,
    /// Function completion.
    Function = 3,
    /// Constructor completion.
    Constructor = 4,
    /// Field completion.
    Field = 5,
    /// Variable completion.
    Variable = 6,
    /// Class completion.
    Class = 7,
    /// Interface completion.
    Interface = 8,
    /// Module completion.
    Module = 9,
    /// Property completion.
    Property = 10,
    /// Unit completion.
    Unit = 11,
    /// Value completion.
    Value = 12,
    /// Enum completion.
    Enum = 13,
    /// Keyword completion.
    Keyword = 14,
    /// Snippet completion.
    Snippet = 15,
    /// Color completion.
    Color = 16,
    /// File completion.
    File = 17,
    /// Reference completion.
    Reference = 18,
    /// Folder completion.
    Folder = 19,
    /// EnumMember completion.
    EnumMember = 20,
    /// Constant completion.
    Constant = 21,
    /// Struct completion.
    Struct = 22,
    /// Event completion.
    Event = 23,
    /// Operator completion.
    Operator = 24,
    /// TypeParameter completion.
    TypeParameter = 25,
}

/// Represents a diagnostic.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// The range of the diagnostic.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub range: Range<usize>,
    /// The severity of the diagnostic.
    pub severity: Option<DiagnosticSeverity>,
    /// The diagnostic's code.
    pub code: Option<String>,
    /// The source of the diagnostic.
    pub source: Option<String>,
    /// The diagnostic's message.
    pub message: String,
}

/// UTF-16 line/character position in protocol wire order (0-based line).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextPosition {
    /// Zero-based line index.
    pub line: u32,
    /// Zero-based UTF-16 character offset within the line.
    pub character: u32,
}

/// UTF-16 text range for protocol projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextRange {
    /// Range start.
    pub start: TextPosition,
    /// Range end.
    pub end: TextPosition,
}

impl TextRange {
    /// Creates a zero-width range at the document origin.
    pub fn empty() -> Self {
        let origin = TextPosition { line: 0, character: 0 };
        Self { start: origin, end: origin }
    }
}

/// A resource location expressed with UTF-16 text coordinates.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TextLocation {
    /// Document URI.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_arc_str"))]
    pub uri: Arc<str>,
    /// Range within the document.
    pub range: TextRange,
}

/// Secondary location attached to a structured diagnostic.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DiagnosticRelatedInformation {
    /// Related resource location.
    pub location: TextLocation,
    /// Related message text.
    pub message: String,
}

/// Structured diagnostic projected to UTF-16 text coordinates for protocol consumers.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StructuredDiagnostic {
    /// Primary diagnostic range.
    pub range: TextRange,
    /// Diagnostic severity.
    pub severity: Option<DiagnosticSeverity>,
    /// Stable diagnostic code.
    pub code: Option<String>,
    /// Emitting component within the origin namespace.
    pub source: Option<String>,
    /// Rendered message text.
    pub message: String,
    /// Additional related locations.
    #[cfg_attr(feature = "serde", serde(skip_serializing_if = "Option::is_none"))]
    pub related_information: Option<Vec<DiagnosticRelatedInformation>>,
}

/// Represents a diagnostic severity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagnosticSeverity {
    /// Reports an error.
    Error = 1,
    /// Reports a warning.
    Warning = 2,
    /// Reports an information.
    Information = 3,
    /// Reports a hint.
    Hint = 4,
}

#[cfg(feature = "serde")]
impl serde::Serialize for DiagnosticSeverity {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(*self as u8)
    }
}

#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for DiagnosticSeverity {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        match u8::deserialize(deserializer)? {
            1 => Ok(Self::Error),
            2 => Ok(Self::Warning),
            3 => Ok(Self::Information),
            4 => Ok(Self::Hint),
            other => Err(serde::de::Error::custom(format!("invalid diagnostic severity: {other}"))),
        }
    }
}

/// Represents semantic tokens.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SemanticTokens {
    /// An optional result ID.
    pub result_id: Option<String>,
    /// The actual semantic token data.
    pub data: Vec<SemanticToken>,
}

/// Represents a selection range.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SelectionRange {
    /// The range of the selection.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub range: Range<usize>,
    /// The parent selection range.
    pub parent: Option<Box<SelectionRange>>,
}

/// Represents parameter information.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ParameterInformation {
    /// The label of the parameter.
    pub label: String,
    /// The documentation of the parameter.
    pub documentation: Option<String>,
}

/// Represents signature information.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureInformation {
    /// The label of the signature.
    pub label: String,
    /// The documentation of the signature.
    pub documentation: Option<String>,
    /// The parameters of the signature.
    pub parameters: Option<Vec<ParameterInformation>>,
    /// The index of the active parameter.
    pub active_parameter: Option<u32>,
}

/// Represents signature help.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SignatureHelp {
    /// The signatures.
    pub signatures: Vec<SignatureInformation>,
    /// The index of the active signature.
    pub active_signature: Option<u32>,
    /// The index of the active parameter.
    pub active_parameter: Option<u32>,
}

/// Represents an inlay hint.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InlayHint {
    /// The position of the inlay hint.
    pub position: SourcePosition,
    /// The label of the inlay hint.
    pub label: String,
    /// The kind of the inlay hint.
    pub kind: Option<InlayHintKind>,
    /// The tooltip of the inlay hint.
    pub tooltip: Option<String>,
    /// Padding before the hint.
    pub padding_left: Option<bool>,
    /// Padding after the hint.
    pub padding_right: Option<bool>,
}

/// Represents an inlay hint kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum InlayHintKind {
    /// Type inlay hint.
    Type = 1,
    /// Parameter inlay hint.
    Parameter = 2,
}

/// Represents a code action.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodeAction {
    /// The title of the code action.
    pub title: String,
    /// The kind of the code action.
    pub kind: Option<String>,
    /// The diagnostics this code action resolves.
    pub diagnostics: Option<Vec<Diagnostic>>,
    /// The workspace edit this code action performs.
    pub edit: Option<WorkspaceEdit>,
    /// A command this code action executes.
    pub command: Option<Command>,
    /// Whether this code action is preferred.
    pub is_preferred: Option<bool>,
    /// Why this code action is disabled.
    pub disabled: Option<CodeActionDisabled>,
}

/// Represents a disabled code action.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CodeActionDisabled {
    /// The reason why the code action is disabled.
    pub reason: String,
}

/// Represents a command.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Command {
    /// The title of the command.
    pub title: String,
    /// The identifier of the command.
    pub command: String,
    /// The arguments of the command.
    pub arguments: Option<Vec<serde_json::Value>>,
}

impl From<UniversalElementRole> for SymbolKind {
    fn from(role: UniversalElementRole) -> Self {
        match role {
            UniversalElementRole::Root => SymbolKind::File,
            UniversalElementRole::Container => SymbolKind::Module,
            UniversalElementRole::Definition => SymbolKind::Function,
            UniversalElementRole::Binding => SymbolKind::Variable,
            UniversalElementRole::Reference => SymbolKind::Variable,
            UniversalElementRole::Typing => SymbolKind::Class,
            UniversalElementRole::Statement => SymbolKind::Function,
            UniversalElementRole::Expression => SymbolKind::Variable,
            UniversalElementRole::Call => SymbolKind::Function,
            UniversalElementRole::Metadata => SymbolKind::Property,
            UniversalElementRole::Attribute => SymbolKind::Property,
            UniversalElementRole::Documentation => SymbolKind::String,
            UniversalElementRole::Value => SymbolKind::Constant,
            UniversalElementRole::Error => SymbolKind::Null,
            _ => SymbolKind::Function,
        }
    }
}

impl From<oak_symbols::SymbolInformation> for WorkspaceSymbol {
    fn from(s: oak_symbols::SymbolInformation) -> Self {
        Self { name: s.name, kind: SymbolKind::from(s.role), location: LocationRange { uri: s.uri, range: s.range }, container_name: s.container_name }
    }
}

/// 把符号信息投影为结构项。
pub(crate) fn structure_item_from_symbol(s: oak_symbols::SymbolInformation) -> StructureItem {
    StructureItem { name: s.name, detail: None, role: s.role, range: s.range.clone(), selection_range: s.range, deprecated: false, children: vec![] }
}
