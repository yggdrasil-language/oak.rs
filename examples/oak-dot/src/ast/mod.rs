#![doc = include_str!("readme.md")]
use core::range::Range;

/// Root node of a DOT file.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DotRoot {
    /// The list of graphs defined in the DOT file.
    pub graphs: Vec<Graph>,
}

impl std::fmt::Display for DotRoot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for graph in &self.graphs {
            write!(f, "{}", graph)?;
        }
        Ok(())
    }
}

/// A graph definition (either a 'graph' or a 'digraph').
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Graph {
    /// Whether the graph is strict (no multiple edges between same nodes).
    pub strict: bool,
    /// The type of the graph (graph or digraph).
    pub graph_type: GraphType,
    /// Optional identifier for the graph.
    pub id: Option<String>,
    /// The list of statements within the graph.
    pub statements: Vec<Statement>,
    /// The source range of this graph definition.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl std::fmt::Display for Graph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.strict {
            write!(f, "strict ")?;
        }
        let gtype = match self.graph_type {
            GraphType::Graph => "graph",
            GraphType::Digraph => "digraph",
        };
        write!(f, "{} ", gtype)?;
        if let Some(id) = &self.id {
            write!(f, "{} ", id)?;
        }
        writeln!(f, "{{")?;
        for stmt in &self.statements {
            write!(f, "    {}", stmt)?;
        }
        writeln!(f, "}}")?;
        Ok(())
    }
}

/// The type of a graph.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GraphType {
    /// An undirected graph.
    Graph,
    /// A directed graph.
    Digraph,
}

/// A statement in a DOT graph.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Statement {
    /// A node definition or modification.
    Node(NodeStatement),
    /// An edge definition.
    Edge(EdgeStatement),
    /// A global attribute setting (graph, node, or edge).
    Attribute(AttributeStatement),
    /// A subgraph definition.
    Subgraph(SubgraphStatement),
    /// An assignment statement (key=value).
    Assignment(AssignmentStatement),
}

impl std::fmt::Display for Statement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Statement::Node(n) => {
                write!(f, "{}", n.node_id.id)?;
                if !n.attributes.is_empty() {
                    write!(f, " [")?;
                    for (i, attr) in n.attributes.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", attr.name)?;
                        if let Some(val) = &attr.value {
                            write!(f, "={}", val)?;
                        }
                    }
                    write!(f, "]")?;
                }
                writeln!(f, ";")
            }
            Statement::Edge(e) => {
                match &e.left {
                    EdgeOperand::Node(id) => write!(f, "{}", id.id)?,
                    EdgeOperand::Subgraph(s) => write!(f, "subgraph {} {{ ... }}", s.id.as_deref().unwrap_or(""))?,
                }
                for (op, target) in &e.edges {
                    let op_str = match op {
                        EdgeOp::Directed => "->",
                        EdgeOp::Undirected => "--",
                    };
                    match target {
                        EdgeOperand::Node(id) => write!(f, " {} {}", op_str, id.id)?,
                        EdgeOperand::Subgraph(s) => write!(f, " {} subgraph {} {{ ... }}", op_str, s.id.as_deref().unwrap_or(""))?,
                    }
                }
                if !e.attributes.is_empty() {
                    write!(f, " [")?;
                    for (i, attr) in e.attributes.iter().enumerate() {
                        if i > 0 {
                            write!(f, ", ")?;
                        }
                        write!(f, "{}", attr.name)?;
                        if let Some(val) = &attr.value {
                            write!(f, "={}", val)?;
                        }
                    }
                    write!(f, "]")?;
                }
                writeln!(f, ";")
            }
            Statement::Attribute(a) => {
                let target = match a.target {
                    AttributeTarget::Graph => "graph",
                    AttributeTarget::Node => "node",
                    AttributeTarget::Edge => "edge",
                };
                write!(f, "{} [", target)?;
                for (i, attr) in a.attributes.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", attr.name)?;
                    if let Some(val) = &attr.value {
                        write!(f, "={}", val)?;
                    }
                }
                writeln!(f, "];")
            }
            Statement::Subgraph(s) => {
                write!(f, "subgraph ")?;
                if let Some(id) = &s.id {
                    write!(f, "{} ", id)?;
                }
                writeln!(f, "{{")?;
                for stmt in &s.statements {
                    write!(f, "        {}", stmt)?;
                }
                writeln!(f, "    }}")
            }
            Statement::Assignment(a) => {
                writeln!(f, "{}={};", a.id, a.value)
            }
        }
    }
}

/// A node statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeStatement {
    /// The ID of the node.
    pub node_id: NodeId,
    /// Attributes associated with the node.
    pub attributes: Vec<Attribute>,
    /// The parsed field structure when `shape` is `record` or `mrecord`.
    ///
    /// The raw `label` attribute stays in `attributes`. This field is the
    /// structural view of the same text, so consumers can address fields and
    /// ports instead of splitting the label themselves.
    pub record: Option<RecordLabel>,
    /// The source range of this node statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A `record` or `mrecord` label, split into fields and brace groups.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordLabel {
    /// Top-level fields and groups in source order.
    pub items: Vec<RecordItem>,
    /// The source range of the label text inside its quotes.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One item of a record label.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RecordItem {
    /// A single field.
    Field(RecordField),
    /// A `{ ... }` brace group that stacks its fields.
    Group(RecordGroup),
}

/// One record field.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordField {
    /// The port name written as `<name>` before the field text.
    pub port: Option<String>,
    /// The field text with escapes resolved and surrounding whitespace trimmed.
    pub text: String,
    /// The source range of this field inside the label text.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A brace group inside a record label.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RecordGroup {
    /// Fields and nested groups in source order.
    pub items: Vec<RecordItem>,
    /// The source range of this group inside the label text, including braces.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// An edge statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EdgeStatement {
    /// The left-hand side operand of the edge.
    pub left: EdgeOperand,
    /// The list of edges following the left operand.
    pub edges: Vec<(EdgeOp, EdgeOperand)>,
    /// Attributes associated with the edge(s).
    pub attributes: Vec<Attribute>,
    /// The source range of this edge statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// An operand in an edge statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeOperand {
    /// A single node ID.
    Node(NodeId),
    /// A subgraph.
    Subgraph(SubgraphStatement),
}

/// An edge operator.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum EdgeOp {
    /// A directed edge operator (->).
    Directed, // ->
    /// An undirected edge operator (--).
    Undirected, // --
}

/// An attribute statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AttributeStatement {
    /// The target of the attribute setting.
    pub target: AttributeTarget,
    /// The list of attributes to set.
    pub attributes: Vec<Attribute>,
    /// The source range of this attribute statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// The target of an attribute statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AttributeTarget {
    /// Affects the graph.
    Graph,
    /// Affects all nodes.
    Node,
    /// Affects all edges.
    Edge,
}

/// A subgraph statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SubgraphStatement {
    /// Optional identifier for the subgraph.
    pub id: Option<String>,
    /// The list of statements within the subgraph.
    pub statements: Vec<Statement>,
    /// The source range of this subgraph statement.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// An assignment statement.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssignmentStatement {
    /// The key (identifier).
    pub id: String,
    /// The value.
    pub value: String,
    /// The source range of this assignment.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A node identifier with optional port and compass point.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NodeId {
    /// The main identifier of the node.
    pub id: String,
    /// Optional port on the node.
    pub port: Option<Port>,
    /// The source range of this node ID.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A port on a node.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Port {
    /// Optional identifier for the port.
    pub id: Option<String>,
    /// Optional compass point.
    pub compass: Option<Compass>,
    /// The source range of this port definition.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Compass points for ports.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Compass {
    /// North.
    N,
    /// North-East.
    NE,
    /// East.
    E,
    /// South-East.
    SE,
    /// South.
    S,
    /// South-West.
    SW,
    /// West.
    W,
    /// North-West.
    NW,
    /// Center.
    C,
}

/// A single attribute (key-value pair).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Attribute {
    /// The name of the attribute.
    pub name: String,
    /// The optional value of the attribute.
    pub value: Option<String>,
    /// The source range of this attribute.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}
