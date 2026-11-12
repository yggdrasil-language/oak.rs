//! Typed AST for MDL connection tables.
//!
//! Every node keeps the authored byte range of its source line, so a consumer
//! can point at the exact text that produced it. Atom and bond indices are
//! **1-based**, exactly as authored in the connection table.

use core::range::Range;

/// Root of a parsed MDL document.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlRoot {
    /// Records in source order: one per molfile or SDF member.
    pub records: Vec<MdlRecord>,
}

/// The connection-table dialect of one record.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MdlVersion {
    /// The fixed-column V2000 layout.
    V2000,
}

/// One molfile or SDF member.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlRecord {
    /// First header line: the record name.
    pub name: Option<String>,
    /// Second header line: program and timestamp.
    pub program: Option<String>,
    /// Third header line: free-form comment.
    pub comment: Option<String>,
    /// The connection-table dialect.
    pub version: MdlVersion,
    /// Atom count declared by the counts line.
    pub atom_count: usize,
    /// Bond count declared by the counts line.
    pub bond_count: usize,
    /// Atoms in declaration order.
    pub atoms: Vec<MdlAtom>,
    /// Bonds in declaration order.
    pub bonds: Vec<MdlBond>,
    /// `M  XXX` property blocks in declaration order.
    pub properties: Vec<MdlProperty>,
    /// `> <NAME>` data fields in declaration order.
    pub data: Vec<MdlDataField>,
    /// Byte range covering the whole record.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One atom line.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlAtom {
    /// Element symbol as authored, for example `C` or `Cl`.
    pub element: String,
    /// Authored x coordinate.
    pub x: f64,
    /// Authored y coordinate.
    pub y: f64,
    /// Authored z coordinate.
    pub z: f64,
    /// Mass difference field, `0` when blank.
    pub mass_difference: i16,
    /// Charge decoded from the charge field, `0` when neutral.
    pub charge: i8,
    /// Stereo parity field as authored.
    pub stereo: i16,
    /// Byte range of the atom line.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Bond order as authored in the connection table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum MdlBondOrder {
    /// Order 1.
    Single,
    /// Order 2.
    Double,
    /// Order 3.
    Triple,
    /// Order 4, an aromatic bond.
    Aromatic,
    /// Any other authored order, kept verbatim.
    Other(u8),
}

/// One bond line.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlBond {
    /// First atom index, 1-based.
    pub from: usize,
    /// Second atom index, 1-based.
    pub to: usize,
    /// Bond order.
    pub order: MdlBondOrder,
    /// Stereo field as authored.
    pub stereo: i16,
    /// Byte range of the bond line.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One entry of a property block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlPropertyEntry {
    /// Atom index the entry applies to, 1-based.
    pub atom: usize,
    /// The authored value, for example a charge or an isotope mass.
    pub value: i16,
}

/// One `M  CHG`, `M  ISO`, `M  RAD`, or alias property block.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlProperty {
    /// Property keyword, for example `CHG`.
    pub kind: String,
    /// Declared entry count.
    pub count: usize,
    /// Parsed atom/value pairs.
    pub entries: Vec<MdlPropertyEntry>,
    /// The authored line, trimmed.
    pub raw: String,
    /// Byte range of the property line.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One `> <NAME>` data field.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MdlDataField {
    /// Field name between the angle brackets.
    pub name: String,
    /// Value lines joined with `\n`.
    pub value: String,
    /// Byte range covering the header and every value line.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}
