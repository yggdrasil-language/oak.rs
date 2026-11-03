//! Abstract syntax tree for the PlantUML class and sequence subset.

use core::{fmt, range::Range};

/// The root of a PlantUML document.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PlantUmlRoot {
    /// Diagrams found inside the document.
    pub diagrams: Vec<PlantUmlDiagram>,
}

impl fmt::Display for PlantUmlRoot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "@startuml")?;
        for diagram in &self.diagrams {
            writeln!(f, "{diagram}")?;
        }
        write!(f, "@enduml")
    }
}

/// A supported PlantUML diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PlantUmlDiagram {
    /// A class diagram.
    Class(ClassDiagram),
    /// A sequence diagram.
    Sequence(SequenceDiagram),
}

impl fmt::Display for PlantUmlDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(diagram) => fmt::Display::fmt(diagram, f),
            Self::Sequence(diagram) => fmt::Display::fmt(diagram, f),
        }
    }
}

/// A class diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDiagram {
    /// Class declarations in source order.
    pub classes: Vec<ClassDeclaration>,
    /// Relations in source order.
    pub relations: Vec<ClassRelation>,
    /// The source span of the diagram.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ClassDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "class diagram")?;
        for class in &self.classes {
            writeln!(f, "{class}")?;
        }
        for relation in &self.relations {
            writeln!(f, "{relation}")?;
        }
        Ok(())
    }
}

/// The keyword used to declare a class-like element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassKeyword {
    /// A plain `class`.
    Class,
    /// An `interface`.
    Interface,
    /// An `abstract class`.
    Abstract,
    /// An `enum`.
    Enum,
}

/// A class, interface, abstract class or enum declaration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDeclaration {
    /// The identifier or the `as` alias. Equals the written name when no alias exists.
    pub name: String,
    /// The quoted display name from `class "Display Name" as Alias`.
    pub display: Option<String>,
    /// The declaration keyword.
    pub keyword: ClassKeyword,
    /// Members declared in the optional block.
    pub members: Vec<ClassMember>,
    /// The source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ClassDeclaration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.keyword {
            ClassKeyword::Class => write!(f, "class ")?,
            ClassKeyword::Interface => write!(f, "interface ")?,
            ClassKeyword::Abstract => write!(f, "abstract class ")?,
            ClassKeyword::Enum => write!(f, "enum ")?,
        }
        if let Some(display) = &self.display {
            write!(f, "\"{display}\" as ")?;
        }
        write!(f, "{}", self.name)?;
        if !self.members.is_empty() {
            writeln!(f, " {{")?;
            for member in &self.members {
                writeln!(f, "    {member}")?;
            }
            write!(f, "}}")?;
        }
        Ok(())
    }
}

/// The visibility of a class member.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MemberVisibility {
    /// Public, written `+`.
    Public,
    /// Private, written `-`.
    Private,
    /// Protected, written `#`.
    Protected,
}

/// One member inside a class member block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassMember {
    /// The member visibility.
    pub visibility: MemberVisibility,
    /// The member text after the visibility marker, trimmed.
    pub text: String,
    /// Whether the member text contains `(`.
    pub is_method: bool,
    /// The source span of the member line.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ClassMember {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let marker = match self.visibility {
            MemberVisibility::Public => '+',
            MemberVisibility::Private => '-',
            MemberVisibility::Protected => '#',
        };
        write!(f, "{marker} {}", self.text)
    }
}

/// The normalized kind of a class relation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ClassRelationKind {
    /// An association (`-->` or `<--`).
    Association,
    /// An inheritance (`<|--` or `--|>`).
    Inheritance,
    /// A composition (`*--` or `--*`).
    Composition,
    /// An aggregation (`o--` or `--o`).
    Aggregation,
    /// A dependency (`..>` or `<..`).
    Dependency,
    /// A plain link (`..`).
    Link,
}

/// A relation between two classes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassRelation {
    /// The left endpoint.
    pub left: String,
    /// The right endpoint.
    pub right: String,
    /// The left multiplicity, without quotes.
    pub left_multiplicity: Option<String>,
    /// The right multiplicity, without quotes.
    pub right_multiplicity: Option<String>,
    /// The normalized relation kind.
    pub kind: ClassRelationKind,
    /// Whether the UML marker is drawn at the left endpoint.
    pub marker_at_left: bool,
    /// The relation label after `:`.
    pub label: Option<String>,
    /// The source span of the relation.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ClassRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arrow = match (self.kind, self.marker_at_left) {
            (ClassRelationKind::Association, true) => "<--",
            (ClassRelationKind::Association, false) => "-->",
            (ClassRelationKind::Inheritance, true) => "<|--",
            (ClassRelationKind::Inheritance, false) => "--|>",
            (ClassRelationKind::Composition, true) => "*--",
            (ClassRelationKind::Composition, false) => "--*",
            (ClassRelationKind::Aggregation, true) => "o--",
            (ClassRelationKind::Aggregation, false) => "--o",
            (ClassRelationKind::Dependency, true) => "<..",
            (ClassRelationKind::Dependency, false) => "..>",
            (ClassRelationKind::Link, _) => "..",
        };

        write!(f, "{}", self.left)?;
        if let Some(multiplicity) = &self.left_multiplicity {
            write!(f, " \"{multiplicity}\"")?;
        }
        write!(f, " {arrow} ")?;
        if let Some(multiplicity) = &self.right_multiplicity {
            write!(f, "\"{multiplicity}\" ")?;
        }
        write!(f, "{}", self.right)?;
        if let Some(label) = &self.label {
            write!(f, " : {label}")?;
        }
        Ok(())
    }
}

/// A sequence diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SequenceDiagram {
    /// Participants in source order.
    pub participants: Vec<Participant>,
    /// Messages in source order.
    pub messages: Vec<SequenceMessage>,
    /// Activation spans closed over message indices.
    pub activations: Vec<ActivationSpan>,
    /// The source span of the diagram.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for SequenceDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "sequence diagram")?;
        for participant in &self.participants {
            writeln!(f, "{participant}")?;
        }
        for message in &self.messages {
            writeln!(f, "{message}")?;
        }
        for activation in &self.activations {
            writeln!(f, "activation {} messages {}..={}", activation.participant, activation.from_message, activation.to_message)?;
        }
        Ok(())
    }
}

/// Whether a participant was declared with `participant` or `actor`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParticipantKind {
    /// A `participant`.
    Participant,
    /// An `actor`.
    Actor,
}

/// A sequence diagram participant.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Participant {
    /// The participant identifier, or the alias when one is present.
    pub id: String,
    /// The quoted display name.
    pub display: Option<String>,
    /// The declaration keyword.
    pub kind: ParticipantKind,
    /// The source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for Participant {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let keyword = match self.kind {
            ParticipantKind::Participant => "participant",
            ParticipantKind::Actor => "actor",
        };
        write!(f, "{keyword} ")?;
        if let Some(display) = &self.display {
            write!(f, "\"{display}\" as ")?;
        }
        write!(f, "{}", self.id)
    }
}

/// The arrow used by a sequence message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessageArrow {
    /// `->`.
    Solid,
    /// `->>`.
    SolidArrow,
    /// `-->`.
    Dotted,
    /// `-->>`.
    DottedArrow,
}

/// One sequence diagram message.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SequenceMessage {
    /// The sending participant.
    pub from: String,
    /// The receiving participant.
    pub to: String,
    /// The optional message text after `:`.
    pub text: Option<String>,
    /// The message arrow.
    pub arrow: MessageArrow,
    /// The source span of the message.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for SequenceMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arrow = match self.arrow {
            MessageArrow::Solid => "->",
            MessageArrow::SolidArrow => "->>",
            MessageArrow::Dotted => "-->",
            MessageArrow::DottedArrow => "-->>",
        };
        write!(f, "{} {arrow} {}", self.from, self.to)?;
        if let Some(text) = &self.text {
            write!(f, " : {text}")?;
        }
        Ok(())
    }
}

/// An activation interval expressed as message indices.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivationSpan {
    /// The activated participant.
    pub participant: String,
    /// The inclusive index of the first message in the activation.
    pub from_message: usize,
    /// The inclusive index of the last message in the activation.
    pub to_message: usize,
}
