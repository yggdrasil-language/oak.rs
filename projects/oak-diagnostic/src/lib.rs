#![feature(new_range_api)]
#![warn(missing_docs)]
//! Diagnostic reporting for the Oak language framework.
//!
//! This crate provides structures and traits for representing and managing
//! diagnostics (errors, warnings, advice) in a way that is compatible with
//! various frontends like LSP or CLI output.
//!
//! Legacy presentation types remain for existing emitters. New integrations should
//! prefer [`unified`] adapters on top of the shared `diagnostic` crate.

use oak_core::{
    errors::{OakError, OakErrorKind},
    source::Source,
};
use oak_vfs::LineMap;

mod unified;

pub use unified::{
    diagnostic_set_from_output, from_oak_error, oak_severity, oak_wire_code,
};
pub use diagnostic::{
    Diagnostic as UnifiedDiagnostic, DiagnosticCode as UnifiedDiagnosticCode,
    DiagnosticLabel as UnifiedDiagnosticLabel, DiagnosticLocation as UnifiedDiagnosticLocation,
    DiagnosticOrigin as UnifiedDiagnosticOrigin, DiagnosticSet, DiagnosticSeverity as UnifiedSeverity,
    DiagnosticSink, Message as UnifiedMessage,
};

/// Severity of a diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Severity {
    /// An error that must be fixed.
    Error,
    /// A warning that should be addressed.
    Warning,
    /// An advice or suggestion for improvement.
    Advice,
}

/// A labeled region in the source code.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Label {
    /// The message associated with the label.
    pub message: Option<String>,
    /// The byte range within the resource.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: core::range::Range<usize>,
    /// The color of the label (optional).
    pub color: Option<String>,
}

/// A diagnostic message.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Diagnostic {
    /// The diagnostic code.
    pub code: Option<String>,
    /// The primary message.
    pub message: String,
    /// The internationalization key.
    pub i18n_key: Option<String>,
    /// The internationalization arguments.
    pub i18n_args: std::collections::HashMap<String, String>,
    /// The severity of the diagnostic.
    pub severity: Severity,
    /// The labeled regions in the source.
    pub labels: Vec<Label>,
    /// A help message providing more details or suggestions.
    pub help: Option<String>,
}

impl Diagnostic {
    /// Creates a new error diagnostic with the given message.
    pub fn error(message: impl Into<String>) -> Self {
        Self { code: None, message: message.into(), i18n_key: None, i18n_args: std::collections::HashMap::new(), severity: Severity::Error, labels: Vec::new(), help: None }
    }

    /// Creates a new warning diagnostic with the given message.
    pub fn warning(message: impl Into<String>) -> Self {
        Self { code: None, message: message.into(), i18n_key: None, i18n_args: std::collections::HashMap::new(), severity: Severity::Warning, labels: Vec::new(), help: None }
    }

    /// Sets the internationalization key for the diagnostic.
    pub fn with_i18n(mut self, key: impl Into<String>) -> Self {
        self.i18n_key = Some(key.into());
        self
    }

    /// Adds an internationalization argument to the diagnostic.
    pub fn with_arg(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.i18n_args.insert(key.into(), value.into());
        self
    }

    /// Creates a diagnostic from a provider and a source.
    pub fn from_provider<P: DiagnosticProvider, S: Source + ?Sized>(provider: &P, source: &S) -> Self {
        provider.to_diagnostic(source)
    }

    /// Adds a labeled region to the diagnostic.
    pub fn with_label(mut self, span: core::range::Range<usize>, message: impl Into<String>) -> Self {
        self.labels.push(Label { message: Some(message.into()), span, color: None });
        self
    }

    /// Adds a help message to the diagnostic.
    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Sets the diagnostic code.
    pub fn with_code(mut self, code: impl Into<String>) -> Self {
        self.code = Some(code.into());
        self
    }
}

impl From<&OakError> for Diagnostic {
    fn from(error: &OakError) -> Self {
        let message = format!("{}", error);
        Diagnostic::error(message)
    }
}

/// A trait for objects that can be converted into a diagnostic.
pub trait DiagnosticProvider {
    /// Converts this object into a diagnostic message using the given source provider.
    fn to_diagnostic<S: Source + ?Sized>(&self, source: &S) -> Diagnostic;
}

/// A provider that emits diagnostics for a language.
pub trait OakDiagnosticsProvider<L: oak_core::Language> {
    /// Emits diagnostics for the given syntax tree and source.
    fn emit_diagnostics<S: Source + ?Sized>(&self, uri: &str, root: &oak_core::tree::RedNode<L>, source: &S) -> Vec<Diagnostic>;
    /// Emits all diagnostics for the given source.
    fn emit_all_diagnostics<S: Source + ?Sized>(&self, uri: &str, source: &S) -> Vec<Diagnostic> {
        let _ = (uri, source);
        Vec::new()
    }
}

impl DiagnosticProvider for OakError {
    fn to_diagnostic<S: Source + ?Sized>(&self, source: &S) -> Diagnostic {
        let kind = self.kind();
        let message = kind.to_string();
        let code = Some(kind.key().to_string());

        let mut diag = Diagnostic::error(message).with_code(code.unwrap()).with_i18n(kind.key());

        match kind {
            OakErrorKind::IoError { .. } => {}
            OakErrorKind::SyntaxError { offset, .. } | OakErrorKind::UnexpectedCharacter { offset, .. } => {
                let start = (*offset).min(source.length());
                let end = (start + 1).min(source.length());
                diag = diag.with_label(core::range::Range { start, end }, "here");
            }
            OakErrorKind::UnexpectedToken { token, offset, .. } => {
                let start = (*offset).min(source.length());
                let end = (start + 1).min(source.length());
                diag = diag.with_label(core::range::Range { start, end }, "here").with_arg("token", token.clone());
            }
            OakErrorKind::ExpectedToken { expected, offset, .. } => {
                let start = (*offset).min(source.length());
                let end = (start + 1).min(source.length());
                diag = diag.with_label(core::range::Range { start, end }, "here").with_arg("expected", expected.clone());
            }
            OakErrorKind::ExpectedName { name_kind, offset, .. } => {
                let start = (*offset).min(source.length());
                let end = (start + 1).min(source.length());
                diag = diag.with_label(core::range::Range { start, end }, "here").with_arg("name_kind", name_kind.clone());
            }
            OakErrorKind::TrailingCommaNotAllowed { offset, .. } => {
                let start = (*offset).min(source.length());
                let end = (start + 1).min(source.length());
                diag = diag.with_label(core::range::Range { start, end }, "here");
            }
            _ => {}
        }

        diag
    }
}

/// A trait for localizing diagnostic messages.
pub trait Localizer {
    /// Localize a message given its key and arguments.
    fn localize(&self, key: &str, args: &std::collections::HashMap<String, String>) -> String;
}

impl Localizer for () {
    fn localize(&self, _key: &str, _args: &std::collections::HashMap<String, String>) -> String {
        String::new()
    }
}

/// A trait for emitting diagnostics.
pub trait Emitter {
    /// Render a diagnostic to a string.
    fn render<S: Source + ?Sized>(&self, source: &S, diagnostic: &Diagnostic) -> String {
        self.render_localized::<S, ()>(source, diagnostic, None, None)
    }

    /// Render a diagnostic to a string with an optional localizer and URI lookup.
    fn render_localized<S: Source + ?Sized, L: Localizer + ?Sized>(&self, source: &S, diagnostic: &Diagnostic, localizer: Option<&L>, uri: Option<&str>) -> String;
}

/// Emitter for ANSI-colored console output.
pub struct ConsoleEmitter {
    /// Whether to use Unicode characters for drawing boxes and lines.
    pub unicode: bool,
}

impl Default for ConsoleEmitter {
    fn default() -> Self {
        Self { unicode: true }
    }
}

impl Emitter for ConsoleEmitter {
    fn render_localized<S: Source + ?Sized, L: Localizer + ?Sized>(&self, source: &S, diagnostic: &Diagnostic, localizer: Option<&L>, uri: Option<&str>) -> String {
        let mut out = String::new();
        let line_map = LineMap::from_source(source);
        let full_text = source.get_text_in(core::range::Range { start: 0, end: source.length() }).into_owned();
        let lines: Vec<&str> = full_text.lines().collect();

        // 1. Header: [code] severity: message or [severity]: message
        let sev_name = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        };
        let sev_color = match diagnostic.severity {
            Severity::Error => "\x1b[31;1m",
            Severity::Warning => "\x1b[33;1m",
            Severity::Advice => "\x1b[36;1m",
        };

        let message = if let (Some(key), Some(loc)) = (&diagnostic.i18n_key, localizer) { loc.localize(key, &diagnostic.i18n_args) } else { diagnostic.message.clone() };

        if let Some(code) = &diagnostic.code {
            out.push_str(&format!("{}[{}]\x1b[0m {}\n", sev_color, code, message));
        }
        else {
            out.push_str(&format!("{}[{}]\x1b[0m {}\n", sev_color, sev_name, message));
        }

        // 2. Snippets
        for label in &diagnostic.labels {
            self.render_snippet(&mut out, source, &line_map, &full_text, &lines, label, uri);
        }

        // 3. Help
        if let Some(help) = &diagnostic.help {
            out.push_str(&format!("\n\x1b[36;1mhelp\x1b[0m: {}\n", help));
        }

        out
    }
}

struct Characters {
    vbar: &'static str,
    hbar: &'static str,
    ltop: &'static str,
    lbot: &'static str,
}

impl Characters {
    fn unicode() -> Self {
        Self { vbar: "│", hbar: "─", ltop: "┌", lbot: "└" }
    }

    fn ascii() -> Self {
        Self { vbar: "|", hbar: "_", ltop: "/", lbot: "|" }
    }
}

impl ConsoleEmitter {
    fn render_snippet<S: Source + ?Sized>(&self, out: &mut String, source: &S, line_map: &LineMap, full_text: &str, lines: &[&str], label: &Label, uri: Option<&str>) {
        let chars = if self.unicode { Characters::unicode() } else { Characters::ascii() };
        let (start_line, _) = line_map.offset_to_line_col_utf16(source, label.span.start);
        let (end_line, _) = line_map.offset_to_line_col_utf16(source, label.span.end);
        let start_line = start_line as usize;
        let end_line = end_line as usize;
        let start_line_start = line_map.line_start(start_line as u32).unwrap_or(0);
        let end_line_start = line_map.line_start(end_line as u32).unwrap_or(0);
        let start_col = full_text.get(start_line_start..label.span.start.min(full_text.len())).unwrap_or("").chars().count();
        let end_col = full_text.get(end_line_start..label.span.end.min(full_text.len())).unwrap_or("").chars().count();

        let line_num_width = (end_line + 1).to_string().len();
        let padding = " ".repeat(line_num_width);

        // Location info: ┌ at url:line:col
        let url_str = uri.unwrap_or("<anonymous>");
        let pos_str = format!("{}:{}", start_line + 1, start_col + 1);
        out.push_str(&format!("  \x1b[34m{}\x1b[0m at {}:{}\n", chars.ltop, url_str, pos_str));
        out.push_str(&format!("{} \x1b[34m{}\x1b[0m\n", padding, chars.vbar));

        if start_line == end_line {
            // Single line label
            if let Some(line_text) = lines.get(start_line) {
                out.push_str(&format!("{:>width$} \x1b[34m{}\x1b[0m {}\n", start_line + 1, chars.vbar, line_text, width = line_num_width));

                let underline_padding = " ".repeat(start_col);
                let underline_len = full_text.get(label.span.start.min(full_text.len())..label.span.end.min(full_text.len())).unwrap_or("").chars().count().max(1);
                let underline = "^".repeat(underline_len);

                let color = label.color.as_deref().unwrap_or("\x1b[31;1m");
                out.push_str(&format!("{} \x1b[34m{}\x1b[0m {}{}{}\x1b[0m", padding, chars.vbar, underline_padding, color, underline));

                if let Some(msg) = &label.message {
                    out.push_str(&format!(" {}\n", msg));
                }
                else {
                    out.push_str("\n");
                }
            }
        }
        else {
            // Multi-line label
            let color = label.color.as_deref().unwrap_or("\x1b[31;1m");
            for i in start_line..=end_line {
                if let Some(line_text) = lines.get(i) {
                    let line_num = i + 1;
                    if i == start_line {
                        out.push_str(&format!("{:>width$} \x1b[34m{}\x1b[0m {}{}{} {}\n", line_num, chars.vbar, color, chars.ltop, "\x1b[0m", line_text, width = line_num_width));
                    }
                    else if i == end_line {
                        out.push_str(&format!("{:>width$} \x1b[34m{}\x1b[0m {}{}{} {}\n", line_num, chars.vbar, color, chars.vbar, "\x1b[0m", line_text, width = line_num_width));

                        let underline_len = end_col.max(1);
                        let underline = "^".repeat(underline_len);
                        out.push_str(&format!("{} \x1b[34m{}\x1b[0m {}{}{}{}", padding, chars.vbar, color, chars.lbot, chars.hbar.repeat(end_col), underline));

                        if let Some(msg) = &label.message {
                            out.push_str(&format!(" {}\x1b[0m\n", msg));
                        }
                        else {
                            out.push_str("\x1b[0m\n");
                        }
                    }
                    else {
                        out.push_str(&format!("{:>width$} \x1b[34m{}\x1b[0m {}{}{} {}\n", line_num, chars.vbar, color, chars.vbar, "\x1b[0m", line_text, width = line_num_width));
                    }
                }
            }
        }
        out.push_str(&format!("{} \x1b[34m{}\x1b[0m\n", padding, chars.vbar));
    }
}

/// Emitter for plain text output without colors.
pub struct PlainTextEmitter {
    /// Whether to use Unicode characters for drawing boxes and lines.
    pub unicode: bool,
}

impl Default for PlainTextEmitter {
    fn default() -> Self {
        Self { unicode: false }
    }
}

impl Emitter for PlainTextEmitter {
    fn render_localized<S: Source + ?Sized, L: Localizer + ?Sized>(&self, source: &S, diagnostic: &Diagnostic, localizer: Option<&L>, uri: Option<&str>) -> String {
        let mut out = String::new();
        let line_map = LineMap::from_source(source);
        let full_text = source.get_text_in(core::range::Range { start: 0, end: source.length() }).into_owned();
        let lines: Vec<&str> = full_text.lines().collect();

        // 1. Header: [code] severity: message or [severity]: message
        let sev_name = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        };

        let message = if let (Some(key), Some(loc)) = (&diagnostic.i18n_key, localizer) { loc.localize(key, &diagnostic.i18n_args) } else { diagnostic.message.clone() };

        if let Some(code) = &diagnostic.code {
            out.push_str(&format!("[{}] {}\n", code, message));
        }
        else {
            out.push_str(&format!("[{}] {}\n", sev_name, message));
        }

        // 2. Snippets
        for label in &diagnostic.labels {
            self.render_snippet(&mut out, source, &line_map, &full_text, &lines, label, uri);
        }

        // 3. Help
        if let Some(help) = &diagnostic.help {
            out.push_str(&format!("\nhelp: {}\n", help));
        }

        out
    }
}

impl PlainTextEmitter {
    fn render_snippet<S: Source + ?Sized>(&self, out: &mut String, source: &S, line_map: &LineMap, full_text: &str, lines: &[&str], label: &Label, uri: Option<&str>) {
        let chars = if self.unicode { Characters::unicode() } else { Characters::ascii() };
        let (start_line, _) = line_map.offset_to_line_col_utf16(source, label.span.start);
        let (end_line, _) = line_map.offset_to_line_col_utf16(source, label.span.end);
        let start_line = start_line as usize;
        let end_line = end_line as usize;
        let start_line_start = line_map.line_start(start_line as u32).unwrap_or(0);
        let end_line_start = line_map.line_start(end_line as u32).unwrap_or(0);
        let start_col = full_text.get(start_line_start..label.span.start.min(full_text.len())).unwrap_or("").chars().count();
        let end_col = full_text.get(end_line_start..label.span.end.min(full_text.len())).unwrap_or("").chars().count();

        let line_num_width = (end_line + 1).to_string().len();
        let padding = " ".repeat(line_num_width);

        // Location info: ┌ at url:line:col
        let url_str = uri.unwrap_or("<anonymous>");
        let pos_str = format!("{}:{}", start_line + 1, start_col + 1);
        out.push_str(&format!("  {} at {}:{}\n", chars.ltop, url_str, pos_str));
        out.push_str(&format!("{} {}\n", padding, chars.vbar));

        if start_line == end_line {
            if let Some(line_text) = lines.get(start_line) {
                out.push_str(&format!("{:>width$} {} {}\n", start_line + 1, chars.vbar, line_text, width = line_num_width));
                let underline_padding = " ".repeat(start_col);
                let underline_len = full_text.get(label.span.start.min(full_text.len())..label.span.end.min(full_text.len())).unwrap_or("").chars().count().max(1);
                let underline = "^".repeat(underline_len);
                out.push_str(&format!("{} {} {}{}", padding, chars.vbar, underline_padding, underline));
                if let Some(msg) = &label.message {
                    out.push_str(&format!(" {}\n", msg));
                }
                else {
                    out.push_str("\n");
                }
            }
        }
        else {
            for i in start_line..=end_line {
                if let Some(line_text) = lines.get(i) {
                    let line_num = i + 1;
                    if i == start_line {
                        out.push_str(&format!("{:>width$} {} {}{}\n", line_num, chars.vbar, chars.ltop, line_text, width = line_num_width));
                    }
                    else if i == end_line {
                        out.push_str(&format!("{:>width$} {} {}{}\n", line_num, chars.vbar, chars.vbar, line_text, width = line_num_width));

                        let underline_len = end_col.max(1);
                        let underline = "^".repeat(underline_len);
                        out.push_str(&format!("{} {} {}{}{}", padding, chars.vbar, chars.lbot, chars.hbar.repeat(end_col), underline));

                        if let Some(msg) = &label.message {
                            out.push_str(&format!(" {}\n", msg));
                        }
                        else {
                            out.push_str("\n");
                        }
                    }
                    else {
                        out.push_str(&format!("{:>width$} {} {}{}\n", line_num, chars.vbar, chars.vbar, line_text, width = line_num_width));
                    }
                }
            }
        }
        out.push_str(&format!("{} {}\n", padding, chars.vbar));
    }
}

/// Emitter for HTML output.
pub struct HtmlEmitter;

impl Emitter for HtmlEmitter {
    fn render_localized<S: Source + ?Sized, L: Localizer + ?Sized>(&self, source: &S, diagnostic: &Diagnostic, localizer: Option<&L>, uri: Option<&str>) -> String {
        let mut out = String::new();
        let line_map = LineMap::from_source(source);
        let full_text = source.get_text_in(core::range::Range { start: 0, end: source.length() }).into_owned();
        let lines: Vec<&str> = full_text.lines().collect();
        let sev_class = match diagnostic.severity {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Advice => "advice",
        };

        let message = if let (Some(key), Some(loc)) = (&diagnostic.i18n_key, localizer) { loc.localize(key, &diagnostic.i18n_args) } else { diagnostic.message.clone() };

        out.push_str("<div class=\"diagnostic\">\n");
        out.push_str(&format!("  <div class=\"header {}\">\n", sev_class));
        if let Some(code) = &diagnostic.code {
            out.push_str(&format!("    <span class=\"code\">[{}]</span> <span class=\"message\">{}</span>\n", code, html_escape(&message)));
        }
        else {
            out.push_str(&format!("    <span class=\"severity\">[{}]</span> <span class=\"message\">{}</span>\n", sev_class, html_escape(&message)));
        }
        out.push_str("  </div>\n");

        for label in &diagnostic.labels {
            self.render_snippet(&mut out, source, &line_map, &full_text, &lines, label, uri);
        }

        if let Some(help) = &diagnostic.help {
            out.push_str(&format!("  <div class=\"help\">help: {}</div>\n", html_escape(help)));
        }
        out.push_str("</div>");

        out
    }
}

impl HtmlEmitter {
    fn render_snippet<S: Source + ?Sized>(&self, out: &mut String, source: &S, line_map: &LineMap, full_text: &str, lines: &[&str], label: &Label, uri: Option<&str>) {
        let (start_line, _) = line_map.offset_to_line_col_utf16(source, label.span.start);
        let (end_line, _) = line_map.offset_to_line_col_utf16(source, label.span.end);
        let start_line = start_line as usize;
        let end_line = end_line as usize;
        let start_line_start = line_map.line_start(start_line as u32).unwrap_or(0);
        let end_line_start = line_map.line_start(end_line as u32).unwrap_or(0);
        let start_col = full_text.get(start_line_start..label.span.start.min(full_text.len())).unwrap_or("").chars().count();
        let end_col = full_text.get(end_line_start..label.span.end.min(full_text.len())).unwrap_or("").chars().count();

        out.push_str("  <div class=\"snippet\">\n");
        let location_prefix = "┌"; // Use the same connector
        let url_str = uri.unwrap_or("<anonymous>");
        let pos_str = format!("{}:{}", start_line + 1, start_col + 1);
        out.push_str(&format!("    <div class=\"location\">  {} at {}:{}</div>\n", location_prefix, html_escape(url_str), pos_str));

        out.push_str("    <pre><code>");
        let line_num_width = (end_line + 1).to_string().len();
        let padding = " ".repeat(line_num_width);

        out.push_str(&format!("<span class=\"padding\">{}</span> <span class=\"vbar\">│</span>\n", padding));

        if start_line == end_line {
            if let Some(line_text) = lines.get(start_line) {
                out.push_str(&format!("<span class=\"line-num\">{: >width$}</span> <span class=\"vbar\">│</span> {}\n", start_line + 1, html_escape(line_text), width = line_num_width));

                let underline_padding = " ".repeat(start_col);
                let underline_len = full_text.get(label.span.start.min(full_text.len())..label.span.end.min(full_text.len())).unwrap_or("").chars().count().max(1);
                let underline = "^".repeat(underline_len);
                out.push_str(&format!("<span class=\"padding\">{}</span> <span class=\"vbar\">│</span> <span class=\"underline\">{}{}</span>", padding, underline_padding, underline));
                if let Some(msg) = &label.message {
                    out.push_str(&format!(" <span class=\"label-msg\">{}</span>", html_escape(msg)));
                }
                out.push_str("\n");
            }
        }
        else {
            for i in start_line..=end_line {
                if let Some(line_text) = lines.get(i) {
                    let line_num = i + 1;
                    if i == start_line {
                        out.push_str(&format!("<span class=\"line-num\">{: >width$}</span> <span class=\"vbar\">│</span> <span class=\"multiline\">┌</span>{}\n", line_num, html_escape(line_text), width = line_num_width));
                    }
                    else if i == end_line {
                        out.push_str(&format!("<span class=\"line-num\">{: >width$}</span> <span class=\"vbar\">│</span> <span class=\"multiline\">│</span>{}\n", line_num, html_escape(line_text), width = line_num_width));

                        let underline_len = end_col.max(1);
                        let underline = "^".repeat(underline_len);
                        out.push_str(&format!("<span class=\"padding\">{}</span> <span class=\"vbar\">│</span> <span class=\"multiline\">└</span><span class=\"underline\">{}</span>", padding, "─".repeat(end_col)));
                        out.push_str(&format!("<span class=\"underline\">{}</span>", underline));
                        if let Some(msg) = &label.message {
                            out.push_str(&format!(" <span class=\"label-msg\">{}</span>", html_escape(msg)));
                        }
                        out.push_str("\n");
                    }
                    else {
                        out.push_str(&format!("<span class=\"line-num\">{: >width$}</span> <span class=\"vbar\">│</span> <span class=\"multiline\">│</span>{}\n", line_num, html_escape(line_text), width = line_num_width));
                    }
                }
            }
        }
        out.push_str(&format!("<span class=\"padding\">{}</span> <span class=\"vbar\">│</span>\n", padding));
        out.push_str("</code></pre>\n");
        out.push_str("  </div>\n");
    }
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;").replace('\'', "&#39;")
}

/// Emitter for LSP (Language Server Protocol) diagnostics.
#[cfg(feature = "serde")]
pub struct LspEmitter;

#[cfg(feature = "serde")]
impl Emitter for LspEmitter {
    fn render_localized<S: Source + ?Sized, L: Localizer + ?Sized>(&self, source: &S, diagnostic: &Diagnostic, localizer: Option<&L>, uri: Option<&str>) -> String {
        let line_map = LineMap::from_source(source);
        let (start_line, start_character, end_line, end_character) = if let Some(label) = diagnostic.labels.first() {
            let (sl, sc) = line_map.offset_to_line_col_utf16(source, label.span.start);
            let (el, ec) = line_map.offset_to_line_col_utf16(source, label.span.end);
            (sl, sc, el, ec)
        }
        else {
            (0, 0, 0, 0)
        };

        let severity = match diagnostic.severity {
            Severity::Error => 1,   // Error
            Severity::Warning => 2, // Warning
            Severity::Advice => 3,  // Information
        };

        let message = if let (Some(key), Some(loc)) = (&diagnostic.i18n_key, localizer) { loc.localize(key, &diagnostic.i18n_args) } else { diagnostic.message.clone() };

        let lsp_diag = serde_json::json!({
            "range": {
                "start": { "line": start_line, "character": start_character },
                "end": { "line": end_line, "character": end_character }
            },
            "severity": severity,
            "code": diagnostic.code.clone().unwrap_or_default(),
            "source": "oak",
            "message": message,
            "relatedInformation": diagnostic.labels.iter().filter_map(|l| {
                l.message.as_ref().map(|msg| {
                    let (sl, sc) = line_map.offset_to_line_col_utf16(source, l.span.start);
                    let (el, ec) = line_map.offset_to_line_col_utf16(source, l.span.end);
                    serde_json::json!({
                        "location": {
                            "uri": uri.unwrap_or(""),
                            "range": {
                                "start": { "line": sl, "character": sc },
                                "end": { "line": el, "character": ec }
                            }
                        },
                        "message": msg.clone()
                    })
                })
            }).collect::<Vec<serde_json::Value>>()
        });

        lsp_diag.to_string()
    }
}
