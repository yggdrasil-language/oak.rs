//! Valkyrie vx parse helpers for AWSL ABI extraction.

use oak_core::{Builder, ParseSession, SourceText};
use oak_valkyrie::ast::ValkyrieRoot;
use oak_valkyrie::{ValkyrieBuilder, ValkyrieLanguage};

/// Parse vx / Valkyrie source via `oak-valkyrie` builder (single authority).
pub fn parse_vx_source(source: &str) -> Result<ValkyrieRoot, String> {
    let language = ValkyrieLanguage::default();
    let builder = ValkyrieBuilder::new(&language);
    let text = SourceText::new(source.to_string());
    let mut session = ParseSession::default();
    builder.build(&text, &[], &mut session).result.map_err(|error| error.to_string())
}

/// Map Oak span to byte range.
pub fn std_range(span: &oak_core::Range<usize>) -> std::ops::Range<usize> {
    span.start..span.end
}
