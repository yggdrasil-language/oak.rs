use oak_core::{ParseSession, Parser, SourceText};

use crate::{VueLanguage, VueParser};

use super::{FormatError, FormatOptions};

/// Validate Vue source while CST layout rules are being implemented.
///
/// Source bytes are preserved without pretty-print finalization.
pub fn format_source(source: &str, _options: &FormatOptions) -> Result<String, FormatError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&text, &[], &mut session);
    parsed.result?;
    if let Some(error) = parsed.diagnostics.into_iter().next() {
        return Err(error);
    }

    Ok(source.to_owned())
}
