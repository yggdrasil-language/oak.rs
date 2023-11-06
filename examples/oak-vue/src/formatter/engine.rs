use oak_core::{ParseSession, Parser, SourceText};

use crate::{VueLanguage, VueParser};

use super::{FormatError, FormatOptions};

/// Format a Vue SFC source string.
///
/// Current implementation validates parse diagnostics and preserves source text
/// (identity output). Region-aware pretty-print is not implemented yet.
pub fn format_source(source: &str, options: &FormatOptions) -> Result<String, FormatError> {
    if source.is_empty() {
        return Ok(String::new());
    }

    let text = SourceText::new(source);
    let language = VueLanguage::default();
    let parser = VueParser::new(&language);
    let mut session = ParseSession::default();
    let parsed = parser.parse(&text, &[], &mut session);

    if parsed.result.is_err() {
        return Err(FormatError::new(format!("oak parse failed: {:?}", parsed.result)));
    }
    if !parsed.diagnostics.is_empty() {
        return Err(FormatError::new(format!("oak diagnostics: {:?}", parsed.diagnostics)));
    }

    Ok(options.finalize_output(source, source.to_string()))
}
