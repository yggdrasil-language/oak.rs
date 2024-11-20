//! Typed AST for ABC notation.
//!
//! Every node keeps the authored byte range. Durations are expressed as a
//! fraction **of a whole note**, so a quarter note is `1/4` regardless of the
//! unit note length the source declared.

use core::range::Range;

/// Root of a parsed ABC document.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcRoot {
    /// Header fields in effect for the whole document.
    pub header: AbcHeader,
    /// Tunes in source order.
    pub tunes: Vec<AbcTune>,
    /// Byte range covering the whole document.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Document header fields.
#[derive(Debug, Clone, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcHeader {
    /// `T:` title.
    pub title: Option<String>,
    /// `C:` composer.
    pub composer: Option<String>,
    /// `Q:` tempo in beats per minute.
    pub tempo_bpm: Option<u32>,
    /// `M:` meter.
    pub meter: Option<AbcMeter>,
    /// `K:` key signature.
    pub key: Option<AbcKey>,
    /// `L:` unit note length.
    pub unit_note_length: Option<AbcFraction>,
    /// Directives and constructs refused before the first tune.
    pub unsupported: Vec<AbcUnsupported>,
    /// Byte range of the header lines.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// A construct the crate refuses to model, kept with its authored text.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcUnsupported {
    /// Short construct name, for example `%%MIDI`, `grace`, or `annotation`.
    pub construct: String,
    /// The authored text of the construct.
    pub source: String,
    /// Byte range of the construct.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One tune, introduced by `X:`.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcTune {
    /// `X:` reference number.
    pub index: Option<u32>,
    /// Voices in source order.
    pub voices: Vec<AbcVoice>,
    /// Constructs refused inside the tune.
    pub unsupported: Vec<AbcUnsupported>,
    /// Byte range of the tune.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One `V:` voice inside a tune.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcVoice {
    /// `V:` identifier, when one was written.
    pub id: Option<String>,
    /// Clef declared on the `V:` line.
    pub clef: Option<AbcClef>,
    /// Measures in source order.
    pub measures: Vec<AbcMeasure>,
    /// Byte range of the voice.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One measure: events up to and including the bar line that closes it.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcMeasure {
    /// Events in time order.
    pub events: Vec<AbcEvent>,
    /// The bar line that closes the measure.
    pub barline: AbcBarline,
    /// Byte range of the measure.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One musical event.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcEvent {
    /// A single note.
    Note(AbcNote),
    /// A rest.
    Rest(AbcRest),
    /// A chord: notes sounding together.
    Chord(AbcChord),
    /// A construct the crate refuses to model.
    Unsupported(AbcUnsupported),
}

/// One note.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcNote {
    /// Pitch.
    pub pitch: AbcPitch,
    /// Duration as a fraction of a whole note.
    pub length: AbcFraction,
    /// Whether the note is tied into the next one.
    pub tie: bool,
    /// Slur attachment.
    pub slur: AbcSlur,
    /// Tuplet membership.
    pub tuplet: Option<AbcTuplet>,
    /// Decorations.
    pub decorations: Vec<AbcDecoration>,
    /// Aligned lyric syllable.
    pub lyric: Option<String>,
    /// Byte range of the note.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One rest.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcRest {
    /// Duration as a fraction of a whole note.
    pub length: AbcFraction,
    /// Byte range of the rest.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One chord: notes sounding together with a shared duration.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcChord {
    /// Member notes.
    pub notes: Vec<AbcNote>,
    /// Shared duration as a fraction of a whole note.
    pub length: AbcFraction,
    /// Byte range of the chord.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// One pitch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcPitch {
    /// Note letter, always uppercase: `C`, `D`, `E`, `F`, `G`, `A`, or `B`.
    pub step: char,
    /// Scientific octave: uppercase letters default to 4, lowercase to 5.
    pub octave: i8,
    /// Explicit accidental.
    pub accidental: Option<AbcAccidental>,
}

/// An accidental.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcAccidental {
    /// `^`.
    Sharp,
    /// `^^`.
    DoubleSharp,
    /// `_`.
    Flat,
    /// `__`.
    DoubleFlat,
    /// `=`.
    Natural,
}

/// A rational duration or unit length.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcFraction {
    /// Numerator.
    pub numerator: u32,
    /// Denominator, never zero.
    pub denominator: u32,
}

impl AbcFraction {
    /// Build a reduced fraction; a zero denominator falls back to `1`.
    pub fn new(numerator: u32, denominator: u32) -> Self {
        let denominator = denominator.max(1);
        let divisor = gcd(numerator, denominator);
        Self { numerator: numerator / divisor, denominator: denominator / divisor }
    }
}

fn gcd(mut a: u32, mut b: u32) -> u32 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a.max(1)
}

/// A meter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcMeter {
    /// `C`, common time.
    Common,
    /// `C|`, cut time.
    Cut,
    /// Explicit beats and unit.
    Explicit {
        /// Beats per measure.
        beats: u8,
        /// Beat unit denominator.
        unit: u8,
    },
}

/// A key signature.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcKey {
    /// Tonic letter, uppercase.
    pub tonic: char,
    /// Tonic accidental.
    pub accidental: Option<AbcAccidental>,
    /// Mode name as authored, lowercase (`m`, `maj`, `dorian`, or empty).
    pub mode: String,
}

/// A clef declared on a `V:` line.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcClef {
    /// `treble`.
    Treble,
    /// `bass`.
    Bass,
    /// `alto`.
    Alto,
    /// Any other authored clef name.
    Other(String),
}

/// A bar line.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcBarline {
    /// `|`.
    #[default]
    Regular,
    /// `||`.
    Double,
    /// `|]`.
    Final,
    /// `|:`.
    RepeatStart,
    /// `:|`.
    RepeatEnd,
    /// `::`.
    RepeatBoth,
}

/// Slur attachment of a note.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcSlur {
    /// No slur.
    None,
    /// Slur begins here.
    Start,
    /// Slur ends here.
    Stop,
    /// Slur both ends here, as in `)(`.
    Both,
}

/// A tuplet declaration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AbcTuplet {
    /// Notes in the tuplet.
    pub actual: u8,
    /// Notes of the same duration the tuplet replaces.
    pub normal: u8,
}

/// A decoration.
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AbcDecoration {
    /// `.` before a note.
    Staccato,
    /// `!tenuto!` or `+tenuto+`.
    Tenuto,
    /// `!accent!` or `+accent+`.
    Accent,
    /// Any other authored decoration name.
    Unknown(String),
}
