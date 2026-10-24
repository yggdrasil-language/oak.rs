use boost_diagnostic::terminal::{SourceID, SourceProvider, SourceSpan, SourceView};

use crate::types::{TextPosition, TextRange};

use super::DiagnosticError;

fn line_text(source: &SourceView<'_>, line_index: usize) -> Result<String, DiagnosticError> {
    source.get_line(line_index).map(|line| line.text).ok_or_else(|| DiagnosticError::IndexTooLarge { given: line_index, max: source.line_count().saturating_sub(1) })
}

fn location_to_position(line_str: &str, line: usize, column: usize, byte_index: usize) -> Result<TextPosition, DiagnosticError> {
    if column > line_str.len() {
        Err(DiagnosticError::ColumnTooLarge { given: column, max: line_str.len() })
    }
    else if !line_str.is_char_boundary(column) {
        Err(DiagnosticError::InvalidCharBoundary { given: byte_index })
    }
    else {
        let character = line_str[..column].encode_utf16().count() as u32;
        Ok(TextPosition { line: line as u32, character })
    }
}

/// Convert a UTF-8 byte index into a protocol text position.
pub fn byte_index_to_position(provider: &impl SourceProvider, file_id: &SourceID, byte_index: usize) -> Result<TextPosition, DiagnosticError> {
    let source = provider.fetch(file_id).map_err(|error| DiagnosticError::Provider(error.to_string()))?;
    let byte_index = u32::try_from(byte_index).map_err(|_| DiagnosticError::IndexTooLarge { given: byte_index, max: source.get_length().saturating_sub(1) })?;

    let (_, line_index, column) = source.get_offset_line(byte_index).ok_or_else(|| DiagnosticError::IndexTooLarge { given: byte_index as usize, max: source.get_length().saturating_sub(1) })?;

    let line_str = line_text(&source, line_index)?;
    location_to_position(&line_str, line_index, column as usize, byte_index as usize)
}

/// Convert a [`SourceSpan`] into a protocol text range.
pub fn byte_span_to_range(provider: &impl SourceProvider, span: &SourceSpan) -> Result<TextRange, DiagnosticError> {
    Ok(TextRange { start: byte_index_to_position(provider, &span.file, span.start as usize)?, end: byte_index_to_position(provider, &span.file, span.end as usize)? })
}

fn character_to_line_offset(line: &str, character: u32) -> Result<usize, DiagnosticError> {
    let line_len = line.len();
    let mut character_offset = 0;

    let mut chars = line.chars();
    while let Some(ch) = chars.next() {
        if character_offset == character {
            let chars_off = chars.as_str().len();
            let ch_off = ch.len_utf8();
            return Ok(line_len - chars_off - ch_off);
        }
        character_offset += ch.len_utf16() as u32;
    }

    if character_offset == character { Ok(line_len) } else { Err(DiagnosticError::ColumnTooLarge { given: character_offset as usize, max: line.len() }) }
}

/// Convert a protocol text position into a UTF-8 byte index.
pub fn position_to_byte_index(provider: &impl SourceProvider, file_id: &SourceID, position: &TextPosition) -> Result<usize, DiagnosticError> {
    let source = provider.fetch(file_id).map_err(|error| DiagnosticError::Provider(error.to_string()))?;
    let line_index = position.line as usize;
    let line = line_text(&source, line_index)?;
    let line_offset = source.line_start(line_index).ok_or_else(|| DiagnosticError::IndexTooLarge { given: line_index, max: source.line_count().saturating_sub(1) })?;
    let byte_offset = character_to_line_offset(&line, position.character)?;
    Ok(line_offset as usize + byte_offset)
}

/// Convert a protocol text range into a UTF-8 byte span.
pub fn range_to_byte_span(provider: &impl SourceProvider, file_id: &SourceID, range: &TextRange) -> Result<std::ops::Range<usize>, DiagnosticError> {
    Ok(position_to_byte_index(provider, file_id, &range.start)?..position_to_byte_index(provider, file_id, &range.end)?)
}

/// Convert a half-open UTF-8 byte range into a protocol text range.
pub fn byte_range_to_text_range(provider: &impl SourceProvider, file_id: &SourceID, start: u64, end: u64) -> Result<TextRange, DiagnosticError> {
    let start_index = usize::try_from(start).map_err(|_| DiagnosticError::RangeOutOfBounds { start, end })?;
    let end_index = usize::try_from(end).map_err(|_| DiagnosticError::RangeOutOfBounds { start, end })?;
    Ok(TextRange { start: byte_index_to_position(provider, file_id, start_index)?, end: byte_index_to_position(provider, file_id, end_index)? })
}
