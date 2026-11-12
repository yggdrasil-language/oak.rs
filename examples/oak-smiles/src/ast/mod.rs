//! Typed AST for SMILES.
//!
//! Atom and bond indices are **0-based** within their own [`SmilesItem`].
//! Every node keeps the authored byte range.

use core::range::Range;

/// Root of a parsed SMILES document.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmilesRoot {
    /// Reactant fragments in source order, split on `.`.
    pub items: Vec<SmilesItem>,
    /// The reaction arrow, when the source is a reaction.
    pub arrow: Option<SmilesArrow>,
    /// Reagent fragments between the two `>` boundaries, split on `.`.
    ///
    /// Reagents keep their own graph instead of surviving only as arrow text,
    /// so a caller can depict a catalyst rather than read its name.
    pub reagents: Vec<SmilesItem>,
    /// Product fragments in source order, empty for a plain molecule.
    pub products: Vec<SmilesItem>,
    /// Byte range covering the whole document.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One dot-separated fragment: a connected molecular graph piece.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmilesItem {
    /// Atoms in traversal order.
    pub atoms: Vec<SmilesAtom>,
    /// Bonds in the order the traversal closed them.
    pub bonds: Vec<SmilesBond>,
    /// Byte range covering the fragment.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Tetrahedral chirality of a bracket atom.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SmilesChirality {
    /// `@`.
    Clockwise,
    /// `@@`.
    CounterClockwise,
}

/// One atom.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmilesAtom {
    /// Element symbol exactly as authored, including aromatic lowercase forms.
    /// `*` marks an author-written wildcard.
    pub element: String,
    /// Whether the atom was written in aromatic lowercase form.
    pub aromatic: bool,
    /// Formal charge, `0` when absent.
    pub charge: i8,
    /// Isotope mass number.
    pub isotope: Option<u16>,
    /// Explicit hydrogen count inside brackets.
    pub explicit_hydrogens: Option<u8>,
    /// Tetrahedral chirality from `@` or `@@`.
    pub chirality: Option<SmilesChirality>,
    /// Byte range of the atom token.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Bond order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SmilesBondOrder {
    /// `-` or implied between two non-aromatic atoms.
    Single,
    /// `=`.
    Double,
    /// `#`.
    Triple,
    /// `:` or implied between two aromatic atoms.
    Aromatic,
    /// `~`, any bond.
    Any,
}

/// Double-bond stereo direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SmilesStereo {
    /// `/`.
    Up,
    /// `\`.
    Down,
}

/// One bond.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmilesBond {
    /// First atom index, 0-based within the item.
    pub from: usize,
    /// Second atom index, 0-based within the item.
    pub to: usize,
    /// Bond order.
    pub order: SmilesBondOrder,
    /// Author-written stereo direction.
    pub stereo: Option<SmilesStereo>,
    /// Byte range covering the construct that declared the bond.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Reaction arrow kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SmilesArrowKind {
    /// A single `>` boundary.
    Single,
    /// A `>>` boundary with no reagent text.
    Double,
}

/// The reaction arrow of a reaction SMILES.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SmilesArrow {
    /// Arrow kind.
    pub kind: SmilesArrowKind,
    /// Reagent text between two `>` boundaries, verbatim.
    pub above: Option<String>,
    /// Reserved for reaction conditions; SMILES carries none.
    pub below: Option<String>,
    /// Byte range covering the arrow and any reagent text.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}
