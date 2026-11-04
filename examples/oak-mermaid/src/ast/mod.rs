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
    /// A state diagram (`stateDiagram` or `stateDiagram-v2`).
    State(StateDiagram),
    /// An entity relationship diagram.
    EntityRelationship(ErDiagram),
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

/// Represents a Mermaid state diagram.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDiagram {
    /// The header version; `stateDiagram` is [`StateDiagramVersion::V1`] and
    /// `stateDiagram-v2` is [`StateDiagramVersion::V2`].
    pub version: StateDiagramVersion,
    /// Top-level states and transitions, in source order.
    pub elements: Vec<StateElement>,
}

/// The header keyword used by a state diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateDiagramVersion {
    /// `stateDiagram`.
    V1,
    /// `stateDiagram-v2`.
    V2,
}

/// A top-level item in a state diagram or a concurrent region.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateElement {
    /// A state declaration, possibly composite.
    State(StateDefinition),
    /// A transition between two states or pseudo states.
    Transition(StateTransition),
}

/// A state, optionally containing concurrent regions.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDefinition {
    /// The identifier used by transitions.
    pub id: String,
    /// The display text from `state "text" as id`.
    pub text: Option<String>,
    /// The state kind derived from a `<<fork>>` style stereotype.
    pub kind: StateKind,
    /// Concurrent regions; a simple state has none and a composite state has one
    /// or more, separated by `--` in the source.
    pub regions: Vec<StateRegion>,
}

/// The kind of a state node.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateKind {
    /// A plain or composite state.
    Normal,
    /// `state fork_state <<fork>>`.
    Fork,
    /// `state join_state <<join>>`.
    Join,
    /// `state choice <<choice>>`.
    Choice,
    /// `state entry <<entryPoint>>`.
    EntryPoint,
    /// `state exit <<exitPoint>>`.
    ExitPoint,
}

/// One concurrent region inside a composite state.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateRegion {
    /// States and transitions written in this region, in source order.
    pub elements: Vec<StateElement>,
}

/// One end of a state transition.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateEndpoint {
    /// A regular state reference.
    State(String),
    /// `[*]` on the source side, the initial pseudo state of the region.
    Start,
    /// `[*]` on the target side, the final pseudo state of the region.
    End,
}

/// A transition between two states.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateTransition {
    /// The source state or pseudo state.
    pub from: StateEndpoint,
    /// The target state or pseudo state.
    pub to: StateEndpoint,
    /// The optional event/guard/action text after `:`.
    pub label: Option<String>,
}

/// Represents a Mermaid entity relationship diagram.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ErDiagram {
    /// Entities in first-appearance order. Entities only referenced by a
    /// relationship are included with an empty attribute list.
    pub entities: Vec<ErEntity>,
    /// Relationships between entities, in source order.
    pub relations: Vec<ErRelation>,
}

/// An entity in an ER diagram.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ErEntity {
    /// The entity name.
    pub name: String,
    /// Attribute lines declared inside the entity block.
    pub attributes: Vec<ErAttribute>,
}

/// One attribute of an ER entity.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ErAttribute {
    /// The declared type, empty when the line only contains a name.
    pub type_name: String,
    /// The attribute name.
    pub name: String,
    /// Primary and foreign key markers.
    pub keys: Vec<ErKey>,
    /// The optional quoted comment.
    pub comment: Option<String>,
}

/// A key marker on an ER attribute.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErKey {
    /// `PK`.
    Primary,
    /// `FK`.
    Foreign,
}

/// A relationship between two ER entities.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ErRelation {
    /// The left entity name.
    pub left: String,
    /// The right entity name.
    pub right: String,
    /// The cardinality at the left end.
    pub left_cardinality: ErCardinality,
    /// The cardinality at the right end.
    pub right_cardinality: ErCardinality,
    /// `true` for a solid `--` line and `false` for a dashed `..` line.
    pub identifying: bool,
    /// The optional relationship label after `:`.
    pub label: Option<String>,
}

/// Crow's Foot cardinality at one end of an ER relationship.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ErCardinality {
    /// `||` — exactly one.
    One,
    /// `|o` / `o|` — zero or one.
    ZeroOrOne,
    /// `|{` / `}|` — one or more.
    OneOrMore,
    /// `o{` / `}o` — zero or more.
    ZeroOrMore,
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

impl std::fmt::Display for StateDiagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let header = match self.version {
            StateDiagramVersion::V1 => "stateDiagram",
            StateDiagramVersion::V2 => "stateDiagram-v2",
        };
        writeln!(f, "{}", header)?;
        write_state_elements(f, &self.elements, 1)
    }
}

/// Writes state elements at the given indentation depth.
fn write_state_elements(f: &mut std::fmt::Formatter<'_>, elements: &[StateElement], depth: usize) -> std::fmt::Result {
    let indent = "    ".repeat(depth);
    for element in elements {
        match element {
            StateElement::State(state) => {
                write!(f, "{}state {}", indent, state.id)?;
                if let Some(text) = &state.text {
                    write!(f, " as {}", text)?;
                }
                match state.kind {
                    StateKind::Normal => {}
                    StateKind::Fork => write!(f, " <<fork>>")?,
                    StateKind::Join => write!(f, " <<join>>")?,
                    StateKind::Choice => write!(f, " <<choice>>")?,
                    StateKind::EntryPoint => write!(f, " <<entryPoint>>")?,
                    StateKind::ExitPoint => write!(f, " <<exitPoint>>")?,
                }
                if state.regions.is_empty() {
                    writeln!(f)?;
                    continue;
                }
                writeln!(f, " {{")?;
                for (index, region) in state.regions.iter().enumerate() {
                    if index > 0 {
                        writeln!(f, "{}{}", indent, "--")?;
                    }
                    write_state_elements(f, &region.elements, depth + 1)?;
                }
                writeln!(f, "{}}}", indent)?;
            }
            StateElement::Transition(transition) => {
                write!(f, "{}", indent)?;
                write_state_endpoint(f, &transition.from)?;
                write!(f, " --> ")?;
                write_state_endpoint(f, &transition.to)?;
                if let Some(label) = &transition.label {
                    write!(f, " : {}", label)?;
                }
                writeln!(f)?;
            }
        }
    }
    Ok(())
}

/// Writes one transition endpoint.
fn write_state_endpoint(f: &mut std::fmt::Formatter<'_>, endpoint: &StateEndpoint) -> std::fmt::Result {
    match endpoint {
        StateEndpoint::State(id) => write!(f, "{}", id),
        StateEndpoint::Start | StateEndpoint::End => write!(f, "[*]"),
    }
}

impl std::fmt::Display for ErDiagram {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "erDiagram")?;
        for entity in &self.entities {
            writeln!(f, "    {} {{", entity.name)?;
            for attribute in &entity.attributes {
                write!(f, "        {}", attribute.type_name)?;
                if !attribute.type_name.is_empty() {
                    write!(f, " ")?;
                }
                write!(f, "{}", attribute.name)?;
                for key in &attribute.keys {
                    write!(
                        f,
                        " {}",
                        match key {
                            ErKey::Primary => "PK",
                            ErKey::Foreign => "FK",
                        }
                    )?;
                }
                if let Some(comment) = &attribute.comment {
                    write!(f, " \"{}\"", comment)?;
                }
                writeln!(f)?;
            }
            writeln!(f, "    }}")?;
        }
        for relation in &self.relations {
            writeln!(
                f,
                "    {} {}{}{} {}{}",
                relation.left,
                er_cardinality_left(relation.left_cardinality),
                if relation.identifying { "--" } else { ".." },
                er_cardinality_right(relation.right_cardinality),
                relation.right,
                relation.label.as_ref().map(|label| format!(" : {label}")).unwrap_or_default()
            )?;
        }
        Ok(())
    }
}

/// Returns the Crow's Foot glyph written before the relationship line.
fn er_cardinality_left(cardinality: ErCardinality) -> &'static str {
    match cardinality {
        ErCardinality::One => "||",
        ErCardinality::ZeroOrOne => "o|",
        ErCardinality::OneOrMore => "}|",
        ErCardinality::ZeroOrMore => "}o",
    }
}

/// Returns the Crow's Foot glyph written after the relationship line.
fn er_cardinality_right(cardinality: ErCardinality) -> &'static str {
    match cardinality {
        ErCardinality::One => "||",
        ErCardinality::ZeroOrOne => "|o",
        ErCardinality::OneOrMore => "|{",
        ErCardinality::ZeroOrMore => "o{",
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
            MermaidDiagram::State(d) => write!(f, "{}", d),
            MermaidDiagram::EntityRelationship(d) => write!(f, "{}", d),
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
