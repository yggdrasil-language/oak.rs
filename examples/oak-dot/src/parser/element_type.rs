//! Element types for the DOT language.
use oak_core::{ElementType, UniversalElementRole};

/// Element types for the DOT language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum DotElementType {
    // Basic kind
    /// An identifier.
    Identifier,
    /// A string literal.
    String,
    /// A number.
    Number,
    /// Whitespace.
    Whitespace,
    /// A newline.
    Newline,

    // DOT keywords
    /// The `graph` keyword.
    Graph,
    /// The `digraph` keyword.
    Digraph,
    /// The `subgraph` keyword.
    Subgraph,
    /// The `node` keyword.
    Node,
    /// The `edge` keyword.
    Edge,
    /// The `strict` keyword.
    Strict,

    // Operators
    /// The `->` arrow operator.
    Arrow, // ->
    /// The `--` line operator.
    Line, // --
    /// The `=` equal operator.
    Equal, // =
    /// The `;` semicolon.
    Semicolon, // ;
    /// The `,` comma.
    Comma, // ,

    // Delimiters
    /// The `{` left brace.
    LeftBrace, // {
    /// The `}` right brace.
    RightBrace, // }
    /// The `[` left bracket.
    LeftBracket, // [
    /// The `]` right bracket.
    RightBracket, // ]
    /// The `(` left paren.
    LeftParen, // (
    /// The `)` right paren.
    RightParen, // )

    // Comments
    /// A comment.
    Comment,

    // Special
    /// The root element.
    Root,
    /// An error element.
    Error,
    /// End of stream marker.
    Eof,
    /// The `:` port separator.
    Colon,

    // Composite elements.
    //
    // These variants follow the token variants one-to-one up to `Colon`: [`DotElementType`]
    // and [`crate::lexer::token_type::DotTokenType`] share a `#[repr(u8)]` layout, and the
    // `From<DotTokenType>` conversion transmutes between them. Every token variant must keep
    // the discriminant of its element counterpart, so new element-only variants may only be
    // added after this prefix and `Colon` must stay at the same position in both enums.
    /// A complete graph definition.
    GraphDefinition,
    /// A node statement.
    GraphNode,
    /// An edge statement.
    GraphEdge,
    /// A `graph`/`node`/`edge` attribute statement.
    GraphAttribute,
    /// A subgraph definition or an anonymous subgraph operand.
    GraphSubgraph,
    /// A bare `key = value` assignment.
    GraphAssignment,
    /// An attribute list enclosed in square brackets.
    AttributeList,
    /// A node identifier with an optional port.
    NodeId,
    /// A port suffix on a node identifier.
    Port,
}

impl ElementType for DotElementType {
    type Role = UniversalElementRole;

    fn role(&self) -> Self::Role {
        match self {
            Self::Root => UniversalElementRole::Root,
            Self::GraphDefinition | Self::GraphSubgraph => UniversalElementRole::Container,
            Self::GraphNode | Self::GraphEdge | Self::GraphAttribute | Self::GraphAssignment => UniversalElementRole::Statement,
            Self::AttributeList => UniversalElementRole::Attribute,
            Self::NodeId => UniversalElementRole::Reference,
            Self::Port => UniversalElementRole::Detail,
            _ => UniversalElementRole::None,
        }
    }
}

impl From<crate::lexer::token_type::DotTokenType> for DotElementType {
    fn from(token: crate::lexer::token_type::DotTokenType) -> Self {
        unsafe { std::mem::transmute(token) }
    }
}
