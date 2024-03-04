#![doc = include_str!("readme.md")]
use core::range::Range;
use oak_core::source::{SourceBuffer, ToSource};
#[cfg(feature = "oak-pretty-print")]
use oak_pretty_print::{AsDocument, Document};

/// MSIL Typed AST Root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MsilTypedRoot<'a> {
    red: oak_core::RedNode<'a, crate::language::MsilLanguage>,
}

impl<'a> oak_core::tree::TypedNode<'a> for MsilTypedRoot<'a> {
    type Language = crate::language::MsilLanguage;

    fn cast(node: oak_core::RedNode<'a, Self::Language>) -> Option<Self> {
        if node.kind::<crate::parser::element_type::MsilElementType>() == crate::parser::element_type::MsilElementType::Root { Some(Self { red: node }) } else { None }
    }

    fn green(&self) -> &oak_core::GreenNode<'a, Self::Language> {
        self.red.green()
    }
}

/// Root node of the MSIL AST.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MsilRoot {
    /// List of items (directives, classes, methods, etc.).
    pub items: Vec<Item>,
}

impl ToSource for MsilRoot {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        for item in &self.items {
            item.to_source(buffer);
            buffer.push("\n")
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for MsilRoot {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::join(self.items.iter().map(|i| i.as_document(&())), Document::Line)
    }
}

/// Top-level items in MSIL.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Item {
    /// Assembly definition.
    Assembly(Assembly),
    /// Module definition.
    Module(String),
    /// Class definition.
    Class(Class),
    /// External assembly reference.
    AssemblyExtern(String),
}

impl ToSource for Item {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            Item::Assembly(a) => a.to_source(buffer),
            Item::Module(m) => {
                buffer.push(".module ");
                buffer.push(m)
            }
            Item::Class(c) => c.to_source(buffer),
            Item::AssemblyExtern(a) => {
                buffer.push(".assembly extern ");
                buffer.push(a);
                buffer.push(" {}")
            }
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Item {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        match self {
            Item::Assembly(a) => a.as_document(&()),
            Item::Module(m) => Document::Text(format!(".module {}", m).into()),
            Item::Class(c) => c.as_document(&()),
            Item::AssemblyExtern(a) => Document::Text(format!(".assembly extern {} {{}}", a).into()),
        }
    }
}

/// Assembly definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Assembly {
    /// Name of the assembly.
    pub name: String,
    /// Span of the assembly definition.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for Assembly {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(".assembly ");
        buffer.push(&self.name);
        buffer.push(" {}")
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Assembly {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::Text(format!(".assembly {} {{}}", self.name).into())
    }
}

/// Class definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Class {
    /// Name of the class.
    pub name: String,
    /// Methods in the class.
    pub methods: Vec<Method>,
    /// Span of the class definition.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for Class {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(".class public auto ansi beforefieldinit ");
        buffer.push(&self.name);
        buffer.push("\n{");
        for method in &self.methods {
            buffer.push("\n");
            method.to_source(buffer)
        }
        buffer.push("\n}")
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Class {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        Document::Concat(vec![
            Document::Text(format!(".class public auto ansi beforefieldinit {}", self.name).into()),
            Document::Line,
            Document::Text("{".into()),
            Document::indent(Document::join(self.methods.iter().map(|m| m.as_document(&())), Document::Line)),
            Document::Text("}".into()),
        ])
    }
}

/// Method definition.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Method {
    /// Name of the method.
    pub name: String,
    /// Instructions in the method.
    pub instructions: Vec<Instruction>,
    /// Span of the method definition.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl ToSource for Method {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        buffer.push(".method public hidebysig static void ");
        buffer.push(&self.name);
        buffer.push("() cil managed\n{");
        if !self.instructions.is_empty() {
            buffer.push("\n    .entrypoint");
            for inst in &self.instructions {
                buffer.push("\n    ");
                inst.to_source(buffer)
            }
        }
        buffer.push("\n}")
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Method {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        let mut body = vec![Document::Text(".entrypoint".into()), Document::Line];
        body.extend(self.instructions.iter().map(|i| i.as_document(&())));

        Document::Concat(vec![
            Document::Text(format!(".method public hidebysig static void {}() cil managed", self.name).into()),
            Document::Line,
            Document::Text("{".into()),
            Document::indent(Document::join(body, Document::Line)),
            Document::Text("}".into()),
        ])
    }
}

/// MSIL instructions.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Instruction {
    /// Simple instruction without parameters.
    Simple(String),
    /// Instruction with a string parameter (e.g., `ldstr`).
    String(String),
    /// Call instruction with a method name.
    Call(String),
}

impl ToSource for Instruction {
    fn to_source(&self, buffer: &mut SourceBuffer) {
        match self {
            Instruction::Simple(s) => buffer.push(s),
            Instruction::String(s) => {
                buffer.push("ldstr \"");
                buffer.push(s);
                buffer.push("\"")
            }
            Instruction::Call(s) => {
                buffer.push("call ");
                buffer.push(s)
            }
        }
    }
}

#[cfg(feature = "oak-pretty-print")]
impl AsDocument for Instruction {
    fn as_document(&self, _config: &Self::Config) -> Document<'_> {
        match self {
            Instruction::Simple(s) => Document::Text(s.clone().into()),
            Instruction::String(s) => Document::Text(format!("ldstr \"{}\"", s).into()),
            Instruction::Call(s) => Document::Text(format!("call {}", s).into()),
        }
    }
}
