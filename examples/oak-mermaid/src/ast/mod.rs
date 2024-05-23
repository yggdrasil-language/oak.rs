use core::range::Range;

/// The root of a Mermaid Abstract Syntax Tree.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MermaidRoot {
    /// The diagrams defined in the Mermaid file.
    pub diagrams: Vec<MermaidDiagram>,
    /// The span of the entire diagram in the source file.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents different types of Mermaid diagrams.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MermaidDiagram {
    /// A flowchart.
    Flowchart(Flowchart),
    /// A sequence diagram.
    Sequence(SequenceDiagram),
    /// A class diagram.
    Class(ClassDiagram),
}

/// Represents a Mermaid flowchart.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Flowchart {
    /// The orientation of the flowchart (e.g., "LR", "TD").
    pub orientation: String,
    /// The nodes and connections in the flowchart.
    pub elements: Vec<FlowElement>,
}

/// Elements in a flowchart.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum FlowElement {
    /// A node definition.
    Node {
        /// The ID of the node.
        id: String,
        /// The optional text/label of the node.
        text: Option<String>,
        /// The shape of the node.
        shape: NodeShape,
    },
    /// An edge between two nodes.
    Edge {
        /// The source node ID.
        from: String,
        /// The target node ID.
        to: String,
        /// The optional text on the edge.
        text: Option<String>,
        /// The style of the edge.
        style: EdgeStyle,
    },
    /// A `subgraph ... end` block.
    Subgraph(Subgraph),
}

/// A Mermaid subgraph block.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Subgraph {
    /// The explicit id from `subgraph id[title]`, if the block declares one.
    pub id: Option<String>,
    /// The human-readable title of the block.
    pub title: String,
    /// Nodes, edges, and nested subgraphs written inside the block, in source order.
    pub elements: Vec<FlowElement>,
}

/// Shapes of nodes in a flowchart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NodeShape {
    /// [Rectangle]
    Box,
    /// (Round edge)
    Round,
    /// ([Stadium])
    Stadium,
    /// [[Subroutine]]
    Subroutine,
    /// [(Cylinder)]
    Cylinder,
    /// ((Circle))
    Circle,
    /// {Rhombus}
    Rhombus,
}

/// Styles of edges in a flowchart.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeStyle {
    /// -->
    Arrow,
    /// ---
    Line,
    /// -.->
    DottedArrow,
    /// ==>
    ThickArrow,
}

/// Represents a Mermaid sequence diagram.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SequenceDiagram {
    /// The participants declared in the sequence diagram, in source order.
    pub participants: Vec<Participant>,
    /// The messages exchanged between participants.
    pub messages: Vec<Message>,
}

/// A participant or actor declared in a sequence diagram.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Participant {
    /// The identifier used by messages.
    pub id: String,
    /// The optional display name from `participant A as Alice`.
    pub display: Option<String>,
    /// Whether the declaration used `participant` or `actor`.
    pub kind: ParticipantKind,
}

/// The keyword used to declare a sequence diagram participant.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ParticipantKind {
    /// `participant`.
    Participant,
    /// `actor`.
    Actor,
}

/// A message in a sequence diagram.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Message {
    /// The sender of the message.
    pub from: String,
    /// The receiver of the message.
    pub to: String,
    /// The content of the message.
    pub text: String,
    /// The type of message arrow.
    pub arrow: MessageArrow,
}

/// Arrow types for sequence diagram messages.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MessageArrow {
    /// ->
    Solid,
    /// -->
    Dotted,
    /// ->>
    SolidArrow,
    /// -->>
    DottedArrow,
}

/// Represents a Mermaid class diagram.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClassDiagram {
    /// The classes defined in the diagram.
    pub classes: Vec<oak_uml::model::Class>,
    /// The relationships between classes.
    pub relations: Vec<oak_uml::model::Relation>,
}

impl std::fmt::Display for MermaidRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for diagram in &self.diagrams {
            write!(f, "{}", diagram)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for SequenceDiagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "sequenceDiagram")?;
        for p in &self.participants {
            let keyword = match p.kind {
                ParticipantKind::Participant => "participant",
                ParticipantKind::Actor => "actor",
            };
            match &p.display {
                Some(display) => writeln!(f, "    {} {} as {}", keyword, p.id, display)?,
                None => writeln!(f, "    {} {}", keyword, p.id)?,
            }
        }
        for m in &self.messages {
            let a = match m.arrow {
                MessageArrow::Solid => "->",
                MessageArrow::Dotted => "-->",
                MessageArrow::SolidArrow => "->>",
                MessageArrow::DottedArrow => "-->>",
            };
            writeln!(f, "    {} {} {}: {}", m.from, a, m.to, m.text)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ClassDiagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "classDiagram")?;
        for class in &self.classes {
            writeln!(f, "    class {}", class.name)?;
            for member in &class.members {
                let symbol = match member.kind {
                    oak_uml::model::MemberKind::Field => "",
                    oak_uml::model::MemberKind::Method => "()",
                };
                writeln!(f, "        {}: {}{}", class.name, member.name, symbol)?;
            }
        }
        for rel in &self.relations {
            let arrow = match rel.kind {
                oak_uml::model::RelationKind::Inheritance => "<|--",
                oak_uml::model::RelationKind::Association => "-->",
                oak_uml::model::RelationKind::Composition => "*--",
                oak_uml::model::RelationKind::Aggregation => "o--",
                oak_uml::model::RelationKind::Dependency => "..>",
            };
            if let Some(label) = &rel.label {
                writeln!(f, "    {} {} {}: {}", rel.source, arrow, rel.target, label)?;
            }
            else {
                writeln!(f, "    {} {} {}", rel.source, arrow, rel.target)?;
            }
        }
        Ok(())
    }
}

impl From<oak_uml::model::UmlDiagram> for ClassDiagram {
    fn from(diagram: oak_uml::model::UmlDiagram) -> Self {
        let mut classes = Vec::new();
        let mut relations = Vec::new();
        for element in diagram.elements {
            match element {
                oak_uml::model::UmlElement::Class(c) => classes.push(c),
                oak_uml::model::UmlElement::Relation(r) => relations.push(r),
                oak_uml::model::UmlElement::Interface(i) => {
                    // Convert interface to class with interface stereotype if needed
                    // For now just convert members
                    let mut c = oak_uml::model::Class::new(i.name);
                    for m in i.members {
                        c = c.with_member(m);
                    }
                    classes.push(c);
                }
            }
        }
        Self { classes, relations }
    }
}

impl std::fmt::Display for MermaidDiagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MermaidDiagram::Flowchart(d) => write!(f, "{}", d),
            MermaidDiagram::Sequence(d) => write!(f, "{}", d),
            MermaidDiagram::Class(d) => write!(f, "{}", d),
        }
    }
}

impl std::fmt::Display for Flowchart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "graph {}", self.orientation)?;
        for element in &self.elements {
            write_flow_element(f, element)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for Subgraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.id {
            Some(id) => writeln!(f, "    subgraph {}[{}]", id, self.title)?,
            None => writeln!(f, "    subgraph {}", self.title)?,
        }
        for element in &self.elements {
            write_flow_element(f, element)?;
        }
        writeln!(f, "    end")
    }
}

/// Writes one flowchart element in Mermaid syntax.
fn write_flow_element(f: &mut std::fmt::Formatter<'_>, element: &FlowElement) -> std::fmt::Result {
    match element {
        FlowElement::Node { id, text, shape } => {
            let (open, close) = match shape {
                NodeShape::Box => ("[", "]"),
                NodeShape::Round => ("(", ")"),
                NodeShape::Stadium => ("([", "])"),
                NodeShape::Subroutine => ("[[", "]]"),
                NodeShape::Cylinder => ("[(", ")]"),
                NodeShape::Circle => ("((", "))"),
                NodeShape::Rhombus => ("{", "}"),
            };
            if let Some(t) = text {
                writeln!(f, "    {}{}{}{}", id, open, t, close)?;
            }
            else {
                writeln!(f, "    {}", id)?;
            }
        }
        FlowElement::Edge { from, to, text, style } => {
            let s = match style {
                EdgeStyle::Arrow => "-->",
                EdgeStyle::Line => "---",
                EdgeStyle::DottedArrow => "-.->",
                EdgeStyle::ThickArrow => "==>",
            };
            if let Some(t) = text {
                writeln!(f, "    {} {}|{}| {}", from, s, t, to)?;
            }
            else {
                writeln!(f, "    {} {} {}", from, s, to)?;
            }
        }
        FlowElement::Subgraph(subgraph) => write!(f, "{}", subgraph)?,
    }
    Ok(())
}
