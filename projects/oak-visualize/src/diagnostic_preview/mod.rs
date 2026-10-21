//! SVG diagnostic preview rendering for structured diagnostics.

use boost_diagnostic::{
    Diagnostic, DiagnosticSet,
    terminal::{Config, SourceCache, SourceRegistry, StructuredRenderError, structured_to_terminal},
};

/// Failure while rendering structured diagnostics to SVG.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RenderError {
    /// Structured diagnostic could not be converted for terminal layout.
    Structured(StructuredRenderError),
    /// Terminal output could not be written.
    Io(String),
    /// Terminal output was not valid UTF-8.
    Utf8,
}

/// Render one structured diagnostic into an SVG preview document.
pub fn structured_to_svg(diagnostic: &Diagnostic, cache: &SourceCache, registry: &SourceRegistry) -> Result<String, RenderError> {
    let terminal = structured_to_terminal(diagnostic, registry, Config::default().with_color(false)).map_err(RenderError::Structured)?;

    let mut buffer = Vec::new();
    terminal.write(cache, &mut buffer).map_err(|error| RenderError::Io(error.to_string()))?;

    let text = String::from_utf8(buffer).map_err(|_| RenderError::Utf8)?;
    Ok(wrap_preview(text))
}

/// Render a diagnostic set into one SVG preview document.
pub fn structured_set_to_svg(set: &DiagnosticSet, cache: &SourceCache, registry: &SourceRegistry) -> Result<String, RenderError> {
    let mut sections = Vec::new();
    for diagnostic in set.diagnostics() {
        sections.push(structured_to_svg(diagnostic, cache, registry)?);
    }
    Ok(sections.join("\n"))
}

fn wrap_preview(text: String) -> String {
    let escaped = html_escape(&text);
    let line_count = escaped.lines().count().max(1);
    let padding = 10;
    let font_size = 12;
    let line_spacing = 3;
    let width = 882;
    let height = padding + line_count * (font_size + line_spacing) + padding;

    format!(
        r#"<svg viewBox="0 0 {width} {height}" xmlns="http://www.w3.org/2000/svg">
  <style>
    pre {{
      background: #1d1f21;
      margin: 0;
      padding: {padding}px;
      border-radius: 6px;
      color: #ffffff;
      font: {font_size}px SFMono-Regular, Consolas, Liberation Mono, Menlo, monospace;
      white-space: pre-wrap;
    }}
  </style>
  <foreignObject x="0" y="0" width="{width}" height="{height}">
    <div xmlns="http://www.w3.org/1999/xhtml">
      <pre>{escaped}</pre>
    </div>
  </foreignObject>
</svg>
"#
    )
}

fn html_escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            value => escaped.push(value),
        }
    }
    escaped
}
