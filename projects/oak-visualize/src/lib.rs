#![doc = include_str!("readme.md")]
#![doc(html_logo_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![doc(html_favicon_url = "https://raw.githubusercontent.com/ygg-lang/oaks/refs/heads/dev/documents/logo.svg")]
#![feature(new_range_api)]
#![warn(missing_docs)]

use std::fmt;

/// SVG preview rendering for structured diagnostics.
pub mod diagnostic_preview;
/// Geometry primitives for visualization.
pub mod geometry;
/// Graph-based visualization structures.
pub mod graph;
/// Layout algorithms for nodes and edges.
pub mod layout;
/// Rendering logic for generating visual output.
pub mod render;
/// Theme configuration for visualization styles.
pub mod theme;
/// Tree-specific visualization tools.
pub mod tree;

/// Error type for oak-visualize operations.
#[derive(Debug)]
pub enum Error {
    /// Layout computation error.
    LayoutError(String),
    /// Rendering error.
    RenderError(String),
    /// Serialization error.
    Serialization(String),
    /// IO error.
    IoError(std::io::Error),
    /// Generic error.
    Generic(String),
}

impl Error {
    /// Creates a generic error with a message.
    pub fn msg<M: Into<String>>(message: M) -> Self {
        Error::Generic(message.into())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::LayoutError(msg) => write!(f, "Layout error: {}", msg),
            Error::RenderError(msg) => write!(f, "Render error: {}", msg),
            Error::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Error::IoError(err) => write!(f, "IO error: {}", err),
            Error::Generic(msg) => write!(f, "Error: {}", msg),
        }
    }
}

impl std::error::Error for Error {}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::IoError(err)
    }
}

/// Result type alias for oak-visualize operations
pub type Result<T> = std::result::Result<T, Error>;

/// Trait for types that can be visualized
pub trait Visualize {
    /// Visualize the object as an SVG string
    fn visualize(&self) -> Result<String>;
}

/// Helper function to visualize a tree node as an SVG string
pub fn to_svg<T: Visualize>(item: &T) -> Result<String> {
    item.visualize()
}

// Re-export commonly used types
pub use crate::{
    geometry::{Point, Rect, Size, Transform},
    graph::{Graph, GraphEdge, GraphLayout, GraphLayoutAlgorithm, GraphLayoutConfig, GraphNode},
    layout::{EdgeType, Layout, LayoutConfig, LayoutEdge, LayoutEngine, LayoutNode, NodeType},
    render::{ElementStyle, ExportFormat, LayoutExporter, RenderConfig, SvgRenderer},
    theme::{ArrowConfig, EdgeTheme, HighlightTheme, NodeTheme, ShadowConfig, TextTheme, VisualizationTheme},
    tree::{TreeLayout, TreeLayoutAlgorithm, TreeLayoutConfig, TreeNode},
};
