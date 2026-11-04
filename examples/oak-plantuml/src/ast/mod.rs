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
    /// An activity diagram, including the old arrow-based syntax.
    Activity(ActivityDiagram),
    /// A state diagram.
    State(StateDiagram),
    /// A component diagram.
    Component(ComponentDiagram),
}

impl fmt::Display for PlantUmlDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Class(diagram) => fmt::Display::fmt(diagram, f),
            Self::Sequence(diagram) => fmt::Display::fmt(diagram, f),
            Self::Activity(diagram) => fmt::Display::fmt(diagram, f),
            Self::State(diagram) => fmt::Display::fmt(diagram, f),
            Self::Component(diagram) => fmt::Display::fmt(diagram, f),
        }
    }
}

/// An activity diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityDiagram {
    /// Activity nodes in source order.
    pub nodes: Vec<ActivityNode>,
    /// The source span of the diagram.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ActivityDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "activity diagram")?;
        for node in &self.nodes {
            writeln!(f, "{node}")?;
        }
        Ok(())
    }
}

/// One node in an activity diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActivityNode {
    /// The initial node, written `start` or `(*)` as a transition source.
    Start,
    /// The final node, written `stop`, `end` or `(*)` as a transition target.
    Stop,
    /// An action, written `:text;`.
    Action(ActivityAction),
    /// An `if` / `elseif` / `else` / `endif` block.
    Decision(ActivityDecision),
    /// A `fork` / `fork again` / `end fork` block.
    Fork(ActivityFork),
    /// A swimlane container opened by `|Name|`.
    Swimlane(ActivitySwimlane),
    /// An arrow transition from the legacy activity syntax.
    Transition(ActivityTransition),
}

impl fmt::Display for ActivityNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start => write!(f, "start"),
            Self::Stop => write!(f, "stop"),
            Self::Action(action) => write!(f, ":{};", action.text),
            Self::Decision(decision) => {
                for branch in &decision.branches {
                    write!(f, "{} ->", branch.kind)?;
                    if let Some(condition) = &branch.condition {
                        write!(f, " ({condition})")?;
                    }
                    writeln!(f)?;
                    for node in &branch.nodes {
                        writeln!(f, "  {node}")?;
                    }
                }
                write!(f, "endif")
            }
            Self::Fork(fork) => {
                for (index, branch) in fork.branches.iter().enumerate() {
                    if index > 0 {
                        writeln!(f, "fork again")?;
                    }
                    for node in &branch.nodes {
                        writeln!(f, "  {node}")?;
                    }
                }
                write!(f, "end fork")
            }
            Self::Swimlane(lane) => write!(f, "|{}|", lane.name),
            Self::Transition(transition) => write!(f, "{} --> {}", transition.from, transition.to),
        }
    }
}

/// A `:text;` action.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityAction {
    /// The action text without the `:` and `;` markers.
    pub text: String,
    /// The source span of the action.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// An `if` / `elseif` / `else` / `endif` decision.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityDecision {
    /// The branch list. The first branch is the `then` branch.
    pub branches: Vec<ActivityBranch>,
    /// The source span of the whole decision.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One branch of a decision.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityBranch {
    /// Whether this branch is `then`, `elseif` or `else`.
    pub kind: ActivityBranchKind,
    /// The condition written in parentheses, when present.
    pub condition: Option<String>,
    /// The guard label written in parentheses after `then` or `else`.
    pub label: Option<String>,
    /// Nodes inside the branch.
    pub nodes: Vec<ActivityNode>,
    /// The source span of the branch.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// The keyword that opens an activity decision branch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActivityBranchKind {
    /// `if (...) then (...)`.
    Then,
    /// `elseif (...) then (...)`.
    ElseIf,
    /// `else (...)`.
    Else,
}

impl fmt::Display for ActivityBranchKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Then => write!(f, "then"),
            Self::ElseIf => write!(f, "elseif"),
            Self::Else => write!(f, "else"),
        }
    }
}

/// A `fork` / `fork again` / `end fork` block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityFork {
    /// Parallel branches. `fork` opens the first branch and `fork again` opens
    /// each following branch.
    pub branches: Vec<ActivityForkBranch>,
    /// The source span of the whole fork block.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One parallel branch of a fork.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityForkBranch {
    /// Nodes inside the branch.
    pub nodes: Vec<ActivityNode>,
    /// The source span of the branch.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A swimlane container opened by `|Name|`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivitySwimlane {
    /// The lane name.
    pub name: String,
    /// The optional `#color` prefix.
    pub color: Option<String>,
    /// Nodes assigned to this lane.
    pub nodes: Vec<ActivityNode>,
    /// The source span of the lane.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One end of a legacy activity transition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ActivityEndpoint {
    /// The initial pseudo node `(*)`.
    Start,
    /// The final pseudo node `(*)`.
    Stop,
    /// A named action or state.
    Named(String),
}

impl fmt::Display for ActivityEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Start | Self::Stop => write!(f, "(*)"),
            Self::Named(name) => write!(f, "{name}"),
        }
    }
}

/// A legacy activity transition such as `(*) --> "First"`.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ActivityTransition {
    /// The source endpoint.
    pub from: ActivityEndpoint,
    /// The target endpoint.
    pub to: ActivityEndpoint,
    /// The optional label after `:`.
    pub label: Option<String>,
    /// The source span of the transition.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A state diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDiagram {
    /// Top-level states and transitions, in source order.
    pub elements: Vec<StateElement>,
    /// The source span of the diagram.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for StateDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "state diagram")?;
        for element in &self.elements {
            writeln!(f, "{element}")?;
        }
        Ok(())
    }
}

/// A top-level item in a state diagram or a concurrent region.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateElement {
    /// A state declaration, possibly composite.
    State(StateDefinition),
    /// A transition between two states or pseudo states.
    Transition(StateTransition),
}

impl fmt::Display for StateElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(state) => write!(f, "state {}", state.id),
            Self::Transition(transition) => write!(f, "{} --> {}", transition.from, transition.to),
        }
    }
}

/// A state, optionally containing concurrent regions.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateDefinition {
    /// The identifier used by transitions.
    pub id: String,
    /// The quoted display text from `state "text" as id`.
    pub text: Option<String>,
    /// The state kind derived from a `<<fork>>` style stereotype.
    pub kind: StateKind,
    /// Concurrent regions. A simple state has none.
    pub regions: Vec<StateRegion>,
}

/// The kind of a state node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
    /// `state start <<start>>`.
    Start,
    /// `state end <<end>>`.
    End,
    /// `state entry <<entryPoint>>`.
    EntryPoint,
    /// `state exit <<exitPoint>>`.
    ExitPoint,
}

/// One concurrent region inside a composite state.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateRegion {
    /// States and transitions written in this region.
    pub elements: Vec<StateElement>,
}

/// One end of a state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum StateEndpoint {
    /// A regular state reference.
    State(String),
    /// `[*]` on the source side.
    Start,
    /// `[*]` on the target side.
    End,
}

impl fmt::Display for StateEndpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::State(id) => write!(f, "{id}"),
            Self::Start | Self::End => write!(f, "[*]"),
        }
    }
}

/// A transition between two states.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct StateTransition {
    /// The source state or pseudo state.
    pub from: StateEndpoint,
    /// The target state or pseudo state.
    pub to: StateEndpoint,
    /// The optional transition text after `:`.
    pub label: Option<String>,
}

/// A component diagram.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentDiagram {
    /// Top-level components, interfaces, ports and packages.
    pub elements: Vec<ComponentElement>,
    /// Relations between components, in source order.
    pub relations: Vec<ComponentRelation>,
    /// The source span of the diagram.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ComponentDiagram {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "component diagram")?;
        for element in &self.elements {
            writeln!(f, "{element}")?;
        }
        for relation in &self.relations {
            writeln!(f, "{relation}")?;
        }
        Ok(())
    }
}

/// One component diagram element.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ComponentElement {
    /// A `component` declaration or a `[Name]` shorthand.
    Component(ComponentNode),
    /// An `interface` declaration.
    Interface(ComponentNode),
    /// A `port`, `portin` or `portout` declaration.
    Port(ComponentPort),
    /// A `package` block containing nested elements.
    Package(ComponentPackage),
}

impl fmt::Display for ComponentElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Component(node) => write!(f, "component {}", node.name),
            Self::Interface(node) => write!(f, "interface {}", node.name),
            Self::Port(port) => write!(f, "{} {}", port.direction, port.name),
            Self::Package(package) => write!(f, "package {}", package.name),
        }
    }
}

/// A component or interface.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentNode {
    /// The identifier used by relations.
    pub name: String,
    /// The quoted display name.
    pub display: Option<String>,
    /// The alias from `as`.
    pub alias: Option<String>,
    /// The source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A port declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentPort {
    /// The port name.
    pub name: String,
    /// The declared direction.
    pub direction: PortDirection,
    /// The source span of the declaration.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// The keyword used to declare a port.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum PortDirection {
    /// `port`.
    InOut,
    /// `portin`.
    In,
    /// `portout`.
    Out,
}

impl fmt::Display for PortDirection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InOut => write!(f, "port"),
            Self::In => write!(f, "portin"),
            Self::Out => write!(f, "portout"),
        }
    }
}

/// A `package` block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentPackage {
    /// The package name.
    pub name: String,
    /// The quoted display name.
    pub display: Option<String>,
    /// The alias from `as`.
    pub alias: Option<String>,
    /// Nested components, interfaces, ports and packages.
    pub elements: Vec<ComponentElement>,
    /// The source span of the package.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A relation between two component diagram nodes.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ComponentRelation {
    /// The left endpoint.
    pub left: String,
    /// The right endpoint.
    pub right: String,
    /// The left multiplicity without quotes.
    pub left_multiplicity: Option<String>,
    /// The right multiplicity without quotes.
    pub right_multiplicity: Option<String>,
    /// The normalized relation kind.
    pub kind: ClassRelationKind,
    /// Whether the UML marker is drawn at the left endpoint.
    pub marker_at_left: bool,
    /// The optional label after `:`.
    pub label: Option<String>,
    /// The source span of the relation.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl fmt::Display for ComponentRelation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let arrow = relation_arrow_text(self.kind, self.marker_at_left);
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

fn relation_arrow_text(kind: ClassRelationKind, marker_at_left: bool) -> &'static str {
    match (kind, marker_at_left) {
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
    /// Package, written `~`.
    Package,
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
            MemberVisibility::Package => '~',
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
