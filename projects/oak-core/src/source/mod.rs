//! Source text management and location tracking for incremental parsing.
//!
//! This module provides structures for managing source code text and tracking
//! locations within it.

use core::range::Range;
use std::borrow::Cow;
mod buffer;
mod cursor;
mod line_map;
mod rope;
mod simd;
mod snapshot;
mod streaming;
mod text;

pub use self::{
    buffer::{SourceBuffer, ToSource},
    cursor::SourceCursor,
    line_map::LineMap,
    rope::{RopeBuffer, RopeSource},
    simd::SimdScanner,
    snapshot::{DocumentSnapshot, SnapshotRevision},
    streaming::{ChunkedBuffer, ChunkedSource},
    text::SourceText,
};
use crate::OakError;

/// A unique identifier for a source file.
pub type SourceId = u32;

/// A chunk of text from a source, including its start offset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TextChunk<'a> {
    /// The start byte offset of this chunk in the source.
    pub start: usize,
    /// The text content of this chunk.
    pub text: &'a str,
}

impl<'a> TextChunk<'a> {
    /// Returns the end byte offset of this chunk.
    #[inline]
    pub fn end(&self) -> usize {
        self.start + self.text.len()
    }

    /// Returns a slice of the chunk text starting from the specified absolute offset.
    #[inline]
    pub fn slice_from(&self, offset: usize) -> &'a str {
        if offset <= self.start {
            return self.text;
        }
        let rel = offset.saturating_sub(self.start);
        self.text.get(rel..).unwrap_or("")
    }
}

/// Represents a text edit operation for incremental updates.
///
/// Text edits are used to apply changes to source text in an incremental
/// manner, such as those received from LSP clients or other text editing
/// operations. Each edit specifies a byte range to replace and the new text
/// to insert in that range.
///
/// # Examples
///
/// ```
/// # #![feature(new_range_api)]
/// # use oak_core::source::TextEdit;
/// use core::range::Range;
/// let edit = TextEdit {
///     span: Range { start: 4, end: 9 }, // Replace characters at positions 4-8
///     text: "world".into(),             // With the text "world"
/// };
/// ```
pub struct TextEdit {
    /// The byte range in the original text to be replaced (start..end)
    pub span: Range<usize>,
    /// The new text to insert in place of the specified range
    pub text: Cow<'static, str>,
}

/// Trait for abstract text sources.
///
/// This trait provides a unified interface for different text sources that may have:
/// - Different character representations (Unicode escapes, HTML entities)
/// - Different internal storage formats (contiguous, chunked, ropes)
/// - Different error handling requirements
///
/// All offsets exposed by this trait are simple byte ranges from the start of this source.
pub trait Source: Send + Sync {
    /// Returns the total length of this source in bytes.
    fn length(&self) -> usize;

    /// Returns a unique identifier for this source, if available.
    ///
    /// Useful for associating diagnostics with specific files.
    fn source_id(&self) -> Option<SourceId> {
        None
    }

    /// Returns a text chunk containing the specified byte offset.
    ///
    /// This allows for efficient traversal of large or non-contiguous sources.
    fn chunk_at(&self, offset: usize) -> TextChunk<'_>;

    /// Returns `true` if the source has no content.
    fn is_empty(&self) -> bool {
        self.length() == 0
    }

    /// Returns the character at the specified byte offset.
    ///
    /// This method should handle any character encoding transformations
    /// and return the actual character that would be seen by the parser.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset from the start of this source.
    ///
    /// # Returns
    ///
    /// The character at the specified offset, or `None` if the offset is invalid.
    fn get_char_at(&self, offset: usize) -> Option<char> {
        self.chunk_at(offset).slice_from(offset).chars().next()
    }

    /// Returns the text content within the specified byte range.
    ///
    /// # Arguments
    ///
    /// * `range` - The byte range to extract text from.
    ///
    /// # Returns
    ///
    /// The text content in the specified range, potentially as a borrowed slice.
    fn get_text_in(&self, range: Range<usize>) -> Cow<'_, str>;

    /// Returns the text from the specified byte offset to the end of the source.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset to start from.
    fn get_text_from(&self, offset: usize) -> Cow<'_, str> {
        if offset >= self.length() {
            return Cow::Borrowed("");
        }
        self.get_text_in(core::range::Range { start: offset, end: self.length() })
    }

    /// Finds the next occurrence of a character starting from an offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset to start searching from.
    /// * `ch` - The character to search for.
    ///
    /// # Returns
    ///
    /// The absolute byte offset of the next occurrence, or `None` if not found.
    fn find_char_from(&self, offset: usize, ch: char) -> Option<usize> {
        let mut cursor = SourceCursor::new_at(self, offset);
        let mut base = offset;
        loop {
            let rest = cursor.rest();
            if let Some(pos) = rest.find(ch) {
                return Some(base + pos);
            }
            let next = cursor.chunk_end();
            if next >= self.length() {
                return None;
            }
            base = next;
            cursor.set_position(next);
        }
    }

    /// Finds the next occurrence of a substring starting from an offset.
    ///
    /// # Arguments
    ///
    /// * `offset` - The byte offset to start searching from.
    /// * `pattern` - The substring to search for.
    ///
    /// # Returns
    ///
    /// The absolute byte offset of the next occurrence, or `None` if not found.
    fn find_str_from(&self, offset: usize, pattern: &str) -> Option<usize> {
        let mut cursor = SourceCursor::new_at(self, offset);
        cursor.find_str(pattern)
    }

    /// Creates a syntax error with location information associated with this source.
    ///
    /// # Arguments
    ///
    /// * `message` - The error message.
    /// * `offset` - The byte offset where the error occurred.
    fn syntax_error(&self, message: String, offset: usize) -> OakError {
        OakError::syntax_error(message, offset, self.source_id())
    }
}

impl Source for str {
    fn length(&self) -> usize {
        self.len()
    }

    fn chunk_at(&self, offset: usize) -> TextChunk<'_> {
        let len = self.len();
        if offset >= len {
            return TextChunk { start: len, text: "" };
        }
        TextChunk { start: offset, text: &self[offset..] }
    }

    fn get_text_in(&self, range: Range<usize>) -> Cow<'_, str> {
        self.get(range.start..range.end).map(Cow::Borrowed).unwrap_or(Cow::Borrowed(""))
    }
}

impl<S: Source + ?Sized> Source for &S {
    fn length(&self) -> usize {
        (**self).length()
    }

    fn chunk_at(&self, offset: usize) -> TextChunk<'_> {
        (**self).chunk_at(offset)
    }

    fn get_text_in(&self, range: Range<usize>) -> Cow<'_, str> {
        (**self).get_text_in(range)
    }

    fn source_id(&self) -> Option<SourceId> {
        (**self).source_id()
    }

    fn get_char_at(&self, offset: usize) -> Option<char> {
        (**self).get_char_at(offset)
    }

    fn get_text_from(&self, offset: usize) -> Cow<'_, str> {
        (**self).get_text_from(offset)
    }

    fn find_char_from(&self, offset: usize, ch: char) -> Option<usize> {
        (**self).find_char_from(offset, ch)
    }

    fn find_str_from(&self, offset: usize, pattern: &str) -> Option<usize> {
        (**self).find_str_from(offset, pattern)
    }

    fn syntax_error(&self, message: String, offset: usize) -> OakError {
        (**self).syntax_error(message, offset)
    }
}
