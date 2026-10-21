use core::range::Range;

use super::Source;

/// A map that tracks line starts in a source file for efficient coordinate conversion.
///
/// `LineMap` provides methods to convert between byte offsets and (line, column) coordinates.
/// It is optimized for cases where multiple conversions are needed for the same source.
#[derive(Debug, Clone)]
pub struct LineMap {
    /// The byte offsets of the start of each line.
    line_starts: Vec<usize>,
    /// The total length of the source in bytes.
    len: usize,
}

impl LineMap {
    /// Creates a new `LineMap` from a source.
    ///
    /// This will scan the entire source to find line endings (`\n`).
    ///
    /// # Examples
    ///
    /// ```
    /// # use oak_core::source::{LineMap, SourceText};
    /// let source = SourceText::new("hello\nworld");
    /// let line_map = LineMap::from_source(&source);
    /// assert_eq!(line_map.line_count(), 2);
    /// ```
    pub fn from_source<S: Source + ?Sized>(source: &S) -> Self {
        let len = source.length();
        let mut line_starts = Vec::new();
        line_starts.push(0);

        let mut offset = 0usize;
        while offset < len {
            let chunk = source.chunk_at(offset);
            let text = chunk.slice_from(offset);
            for (i, b) in text.as_bytes().iter().enumerate() {
                if *b == b'\n' {
                    let next = offset + i + 1;
                    if next <= len {
                        line_starts.push(next)
                    }
                }
            }
            offset = chunk.end()
        }

        Self { line_starts, len }
    }

    /// Returns the total number of lines in the source.
    pub fn line_count(&self) -> usize {
        self.line_starts.len()
    }

    /// Returns the byte offset of the start of the given line (0-indexed).
    pub fn line_start(&self, line: u32) -> Option<usize> {
        self.line_starts.get(line as usize).copied()
    }

    /// Returns the byte offset of the end of the given line (0-indexed).
    ///
    /// The end of the line includes the line ending character(s) if present,
    /// except for the last line which ends at the end of the source.
    pub fn line_end(&self, line: u32) -> Option<usize> {
        let idx = line as usize;
        let start = *self.line_starts.get(idx)?;
        let next = self.line_starts.get(idx + 1).copied().unwrap_or(self.len);
        Some(next.max(start))
    }

    /// Converts a byte offset to (line, column) coordinates using UTF-16 for the column.
    pub fn offset_to_line_col_utf16<S: Source + ?Sized>(&self, source: &S, offset: usize) -> (u32, u32) {
        let offset = offset.min(self.len);
        let line_idx = match self.line_starts.binary_search(&offset) {
            Ok(i) => i,
            Err(0) => 0,
            Err(i) => i - 1,
        };
        let line_start = self.line_starts[line_idx];
        let slice = source.get_text_in(Range { start: line_start, end: offset });
        let col = slice.as_ref().encode_utf16().count() as u32;
        (line_idx as u32, col)
    }

    /// Converts (line, column) coordinates (in UTF-16) to a byte offset.
    pub fn line_col_utf16_to_offset<S: Source + ?Sized>(&self, source: &S, line: u32, col_utf16: u32) -> usize {
        let Some(line_start) = self.line_start(line)
        else {
            return self.len;
        };
        let line_end = self.line_end(line).unwrap_or(self.len);
        let slice = source.get_text_in(Range { start: line_start, end: line_end });
        let text = slice.as_ref();
        let target = col_utf16 as usize;

        let mut utf16 = 0usize;
        for (byte_idx, ch) in text.char_indices() {
            if utf16 >= target {
                return (line_start + byte_idx).min(self.len);
            }
            utf16 += ch.len_utf16()
        }
        line_end.min(self.len)
    }
}
