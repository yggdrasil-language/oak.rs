#![doc = include_str!("readme.md")]
use oak_core::source::{SourceBuffer, ToSource};
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::Document;
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::to_doc::AsDocument;
use std::{string::String, vec::Vec};

/// JASM root node.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JasmRoot {
    /// The class definition.
    pub class: JasmClass,
}

impl ToSource for JasmRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        self.class.to_source(buffer)
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JasmRoot {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        self.class.as_document(config)
    }
}

/// AST node for a JASM class declaration.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JasmClass {
    /// Access modifiers (public, private, etc.).
    pub modifiers: Vec<String>,
    /// Class name.
    pub name: String,
    /// Version information (e.g., 65:0).
    pub version: Option<String>,
    /// List of methods.
    pub methods: Vec<JasmMethod>,
    /// List of fields.
    pub fields: Vec<JasmField>,
    /// Source file information.
    pub source_file: Option<String>,
    /// Super class information.
    pub super_class: Option<String>,
    /// List of implemented interfaces.
    pub interfaces: Vec<String>,
    /// List of annotations.
    pub annotations: Vec<String>,
    /// List of attributes.
    pub attributes: Vec<String>,
}

impl ToSource for JasmClass {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        if let Some(source) = &self.source_file {
            buffer.push(".source ");
            buffer.push(source);
            buffer.push("\n")
        }
        if let Some(super_class) = &self.super_class {
            buffer.push(".super ");
            buffer.push(super_class);
            buffer.push("\n")
        }
        buffer.push(".class ");
        for modifier in &self.modifiers {
            buffer.push(modifier);
            buffer.push(" ")
        }
        buffer.push(&self.name);
        buffer.push("\n");
        if let Some(version) = &self.version {
            buffer.push(".version ");
            buffer.push(version);
            buffer.push("\n")
        }
        for annotation in &self.annotations {
            buffer.push(annotation);
            buffer.push("\n")
        }
        for attribute in &self.attributes {
            buffer.push(attribute);
            buffer.push("\n")
        }
        for interface in &self.interfaces {
            buffer.push(".interface ");
            buffer.push(interface);
            buffer.push("\n")
        }
        buffer.push("\n");
        for field in &self.fields {
            field.to_source(buffer);
            buffer.push("\n")
        }
        for method in &self.methods {
            method.to_source(buffer);
            buffer.push("\n")
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JasmClass {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        let mut docs = Vec::new();
        if let Some(source) = &self.source_file {
            docs.push(Document::Text(format!(".source {}\n", source).into()))
        }
        if let Some(super_class) = &self.super_class {
            docs.push(Document::Text(format!(".super {}\n", super_class).into()))
        }
        let mut class_line = vec![Document::Text(".class ".into())];
        for modifier in &self.modifiers {
            class_line.push(Document::Text(modifier.clone().into()));
            class_line.push(Document::Text(" ".into()))
        }
        class_line.push(Document::Text(self.name.clone().into()));
        docs.push(Document::Concat(class_line));
        docs.push(Document::Line);

        if let Some(version) = &self.version {
            docs.push(Document::Text(format!(".version {}\n", version).into()))
        }
        for annotation in &self.annotations {
            docs.push(Document::Text(format!("{}\n", annotation).into()))
        }
        for attribute in &self.attributes {
            docs.push(Document::Text(format!("{}\n", attribute).into()))
        }
        for interface in &self.interfaces {
            docs.push(Document::Text(format!(".interface {}\n", interface).into()))
        }
        docs.push(Document::Line);

        for field in &self.fields {
            docs.push(field.as_document(config));
            docs.push(Document::Line)
        }
        for method in &self.methods {
            docs.push(method.as_document(config));
            docs.push(Document::Line)
        }
        Document::Concat(docs)
    }
}

/// AST node for a JASM method declaration.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JasmMethod {
    /// Access modifiers (public, static, etc.).
    pub modifiers: Vec<String>,
    /// Method name and type descriptor (e.g., "main":"([Ljava/lang/String)V").
    pub name_and_descriptor: String,
    /// Stack size.
    pub stack_size: Option<u32>,
    /// Number of local variables.
    pub locals_count: Option<u32>,
    /// List of instructions.
    pub instructions: Vec<JasmInstruction>,
    /// List of exception handlers.
    pub exception_handlers: Vec<String>,
    /// List of annotations.
    pub annotations: Vec<String>,
    /// List of attributes.
    pub attributes: Vec<String>,
}

impl ToSource for JasmMethod {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(".method ");
        for modifier in &self.modifiers {
            buffer.push(modifier);
            buffer.push(" ")
        }
        buffer.push(&self.name_and_descriptor);
        buffer.push("\n");
        for annotation in &self.annotations {
            buffer.push("    ");
            buffer.push(annotation);
            buffer.push("\n")
        }
        for attribute in &self.attributes {
            buffer.push("    ");
            buffer.push(attribute);
            buffer.push("\n")
        }
        if let Some(stack) = self.stack_size {
            buffer.push("    .limit stack ");
            buffer.push(&stack.to_string());
            buffer.push("\n")
        }
        if let Some(locals) = self.locals_count {
            buffer.push("    .limit locals ");
            buffer.push(&locals.to_string());
            buffer.push("\n")
        }
        for exception_handler in &self.exception_handlers {
            buffer.push("    ");
            buffer.push(exception_handler);
            buffer.push("\n")
        }
        for inst in &self.instructions {
            buffer.push("    ");
            inst.to_source(buffer);
            buffer.push("\n")
        }
        buffer.push(".end method")
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JasmMethod {
    fn as_document(&self, config: &Self::Config) -> Document<'_> {
        let mut docs = Vec::new();
        let mut method_line = vec![Document::Text(".method ".into())];
        for modifier in &self.modifiers {
            method_line.push(Document::Text(modifier.clone().into()));
            method_line.push(Document::Text(" ".into()))
        }
        method_line.push(Document::Text(self.name_and_descriptor.clone().into()));
        docs.push(Document::Concat(method_line));
        docs.push(Document::Line);

        let mut body = Vec::new();
        for annotation in &self.annotations {
            body.push(Document::Text(format!("{}\n", annotation).into()))
        }
        for attribute in &self.attributes {
            body.push(Document::Text(format!("{}\n", attribute).into()))
        }
        if let Some(stack) = self.stack_size {
            body.push(Document::Text(format!(".limit stack {}\n", stack).into()))
        }
        if let Some(locals) = self.locals_count {
            body.push(Document::Text(format!(".limit locals {}\n", locals).into()))
        }
        for exception_handler in &self.exception_handlers {
            body.push(Document::Text(format!("{}\n", exception_handler).into()))
        }
        for inst in &self.instructions {
            body.push(inst.as_document(config));
            body.push(Document::Line)
        }

        docs.push(Document::indent(Document::Concat(body)));
        docs.push(Document::Text(".end method".into()));
        Document::Concat(docs)
    }
}

/// AST node for a JASM field declaration.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct JasmField {
    /// Access modifiers (public, static, etc.).
    pub modifiers: Vec<String>,
    /// Field name and type descriptor (e.g., "value":"I").
    pub name_and_descriptor: String,
    /// List of annotations.
    pub annotations: Vec<String>,
    /// List of attributes.
    pub attributes: Vec<String>,
}

impl ToSource for JasmField {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        for annotation in &self.annotations {
            buffer.push(annotation);
            buffer.push("\n");
            buffer.push("    ")
        }
        for attribute in &self.attributes {
            buffer.push(attribute);
            buffer.push("\n");
            buffer.push("    ")
        }
        buffer.push(".field ");
        for modifier in &self.modifiers {
            buffer.push(modifier);
            buffer.push(" ")
        }
        buffer.push(&self.name_and_descriptor)
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JasmField {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        let mut docs = Vec::new();
        for annotation in &self.annotations {
            docs.push(Document::Text(format!("{}\n", annotation).into()));
            docs.push(Document::Text("    ".into()))
        }
        for attribute in &self.attributes {
            docs.push(Document::Text(format!("{}\n", attribute).into()));
            docs.push(Document::Text("    ".into()))
        }
        docs.push(Document::Text(".field ".into()));
        for modifier in &self.modifiers {
            docs.push(Document::Text(modifier.clone().into()));
            docs.push(Document::Text(" ".into()))
        }
        docs.push(Document::Text(self.name_and_descriptor.clone().into()));
        Document::Concat(docs)
    }
}

/// AST node for a JASM instruction.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum JasmInstruction {
    /// Simple instruction (e.g., aload_0, return).
    Simple(String),
    /// Instruction with an argument (e.g., ldc "Hello").
    WithArgument {
        /// The instruction name.
        instruction: String,
        /// The instruction argument.
        argument: String,
    },
    /// Method call instruction (e.g., invokespecial Method java/lang/Object."<init>":"()V").
    MethodCall {
        /// The instruction name.
        instruction: String,
        /// The method reference.
        method_ref: String,
    },
    /// Field access instruction (e.g., getstatic Field java/lang/System.out:"Ljava/io/PrintStream;").
    FieldAccess {
        /// The instruction name.
        instruction: String,
        /// The field reference.
        field_ref: String,
    },
}

impl ToSource for JasmInstruction {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            JasmInstruction::Simple(s) => buffer.push(s),
            JasmInstruction::WithArgument { instruction, argument } => {
                buffer.push(instruction);
                buffer.push(" ");
                buffer.push(argument)
            }
            JasmInstruction::MethodCall { instruction, method_ref } => {
                buffer.push(instruction);
                buffer.push(" ");
                buffer.push(method_ref)
            }
            JasmInstruction::FieldAccess { instruction, field_ref } => {
                buffer.push(instruction);
                buffer.push(" ");
                buffer.push(field_ref)
            }
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for JasmInstruction {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        match self {
            JasmInstruction::Simple(s) => Document::Text(s.clone().into()),
            JasmInstruction::WithArgument { instruction, argument } => Document::Concat(vec![Document::Text(instruction.clone().into()), Document::Text(" ".into()), Document::Text(argument.clone().into())]),
            JasmInstruction::MethodCall { instruction, method_ref } => Document::Concat(vec![Document::Text(instruction.clone().into()), Document::Text(" ".into()), Document::Text(method_ref.clone().into())]),
            JasmInstruction::FieldAccess { instruction, field_ref } => Document::Concat(vec![Document::Text(instruction.clone().into()), Document::Text(" ".into()), Document::Text(field_ref.clone().into())]),
        }
    }
}
