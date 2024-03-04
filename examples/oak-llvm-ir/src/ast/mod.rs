#![doc = include_str!("readme.md")]
use oak_core::{
    Range,
    source::{SourceBuffer, ToSource},
};
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::document::Document;
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::to_doc::AsDocument;

/// Root node of an LLVM IR module.
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LLirRoot {
    /// Items in the module (functions, globals, etc.).
    pub items: Vec<LLirItem>,
    /// The span of the root node in the source text.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for LLirRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        for (i, item) in self.items.iter().enumerate() {
            if i > 0 {
                buffer.push("\n");
            }
            item.to_source(buffer);
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirRoot {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        Document::join(self.items.iter().map(|i| i.as_document(config)), Document::Line)
    }
}

/// Represents a top-level item in an LLVM IR module.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LLirItem {
    /// A function definition or declaration.
    Function(LLirFunction),
    /// A global variable definition.
    Global(LLirGlobal),
}

impl ToSource for LLirItem {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            Self::Function(f) => f.to_source(buffer),
            Self::Global(g) => g.to_source(buffer),
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirItem {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        match self {
            Self::Function(f) => f.as_document(config),
            Self::Global(g) => g.as_document(config),
        }
    }
}

/// Represents an LLVM IR function.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LLirFunction {
    /// The name of the function.
    pub name: String,
    /// The return type of the function.
    pub return_type: String,
    /// The parameters of the function.
    pub parameters: Vec<LLirParameter>,
    /// The basic blocks that make up the function body.
    pub blocks: Vec<LLirBlock>,
    /// The span of the function in the source text.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for LLirFunction {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("define ");
        buffer.push(&self.return_type);
        buffer.push(" @");
        buffer.push(&self.name);
        buffer.push("(");
        for (i, param) in self.parameters.iter().enumerate() {
            if i > 0 {
                buffer.push(", ");
            }
            param.to_source(buffer);
        }
        buffer.push(") {\n");
        for block in &self.blocks {
            block.to_source(buffer);
        }
        buffer.push("}\n");
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirFunction {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        let mut parts = vec![Document::text("define "), Document::text(&self.return_type), Document::text(" @"), Document::text(&self.name), Document::text("(")];

        let params_docs = Document::join(self.parameters.iter().map(|p| p.as_document(config)), Document::text(", "));
        parts.push(params_docs);
        parts.push(Document::text(") {"));

        let blocks = Document::join(self.blocks.iter().map(|b| b.as_document(config)), Document::Line);
        parts.push(Document::indent(Document::concat(vec![Document::Line, blocks])));
        parts.push(Document::Line);
        parts.push(Document::text("}"));

        Document::group(Document::concat(parts))
    }
}

/// Represents a parameter in an LLVM IR function.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LLirParameter {
    /// The type of the parameter.
    pub ty: String,
    /// The name of the parameter.
    pub name: String,
}

impl ToSource for LLirParameter {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(&self.ty);
        buffer.push(" %");
        buffer.push(&self.name);
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirParameter {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::concat(vec![Document::text(&self.ty), Document::text(" %"), Document::text(&self.name)])
    }
}

/// Represents a basic block in an LLVM IR function.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LLirBlock {
    /// The optional label for the block.
    pub label: Option<String>,
    /// The instructions contained in the block.
    pub instructions: Vec<LLirInstruction>,
}

impl ToSource for LLirBlock {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        if let Some(label) = &self.label {
            buffer.push(label);
            buffer.push(":\n");
        }
        for inst in &self.instructions {
            buffer.push("  ");
            inst.to_source(buffer);
            buffer.push("\n");
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirBlock {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        let mut parts = Vec::new();
        if let Some(label) = &self.label {
            parts.push(Document::text(label));
            parts.push(Document::text(":"));
            parts.push(Document::Line);
        }
        let insts = Document::join(self.instructions.iter().map(|i| i.as_document(config)), Document::Line);
        parts.push(Document::indent(insts));
        Document::concat(parts)
    }
}

/// Represents an LLVM IR instruction.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LLirInstruction {
    /// The optional result register name.
    pub result: Option<String>,
    /// The instruction opcode.
    pub opcode: String,
    /// The operands for the instruction.
    pub operands: Vec<String>,
}

impl ToSource for LLirInstruction {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        if let Some(res) = &self.result {
            buffer.push("%");
            buffer.push(res);
            buffer.push(" = ");
        }
        buffer.push(&self.opcode);
        buffer.push(" ");
        for (i, op) in self.operands.iter().enumerate() {
            if i > 0 {
                buffer.push(", ");
            }
            buffer.push(op);
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirInstruction {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        let mut parts = Vec::new();
        if let Some(res) = &self.result {
            parts.push(Document::text("%"));
            parts.push(Document::text(res));
            parts.push(Document::text(" = "));
        }
        parts.push(Document::text(&self.opcode));
        parts.push(Document::text(" "));
        let operands = Document::join(self.operands.iter().map(|o| Document::text(o)), Document::text(", "));
        parts.push(operands);
        Document::group(Document::concat(parts))
    }
}

/// Represents an LLVM IR global variable.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LLirGlobal {
    /// The name of the global variable.
    pub name: String,
    /// The type of the global variable.
    pub ty: String,
    /// The initial value of the global variable.
    pub value: String,
    /// Whether the global variable is constant.
    pub is_constant: bool,
}

impl ToSource for LLirGlobal {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push("@");
        buffer.push(&self.name);
        buffer.push(" = ");
        if self.is_constant {
            buffer.push("constant ");
        }
        else {
            buffer.push("global ");
        }
        buffer.push(&self.ty);
        buffer.push(" ");
        buffer.push(&self.value);
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for LLirGlobal {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        let mut parts = vec![Document::text("@"), Document::text(&self.name), Document::text(" = ")];
        if self.is_constant {
            parts.push(Document::text("constant "));
        }
        else {
            parts.push(Document::text("global "));
        }
        parts.push(Document::text(&self.ty));
        parts.push(Document::text(" "));
        parts.push(Document::text(&self.value));
        Document::group(Document::concat(parts))
    }
}
