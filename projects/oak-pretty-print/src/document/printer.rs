use crate::document::Document;
use alloc::{borrow::Cow, string::String};

/// Indent style
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IndentStyle {
    /// Use spaces
    Spaces(u8),
    /// Use tabs
    Tabs,
}

impl Default for IndentStyle {
    fn default() -> Self {
        IndentStyle::Spaces(4)
    }
}

/// Line ending
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum LineEnding {
    /// Unix style (\n)
    Unix,
    /// Windows style (\r\n)
    Windows,
    /// Auto detect
    Auto,
}

impl Default for LineEnding {
    fn default() -> Self {
        LineEnding::Auto
    }
}

/// Printer configuration
#[derive(Debug, Clone)]
pub struct PrinterConfig {
    /// Indent style
    pub indent_style: IndentStyle,
    /// Indent text (cached single-level indent string)
    pub indent_text: Cow<'static, str>,
    /// Line ending
    pub line_ending: LineEnding,
    /// Maximum line length
    pub max_width: usize,
    /// Whether to insert a final newline at the end of the file
    pub insert_final_newline: bool,
    /// Whether to trim trailing whitespace
    pub trim_trailing_whitespace: bool,
    /// Indent size (used for column calculation)
    pub indent_size: usize,
}

impl Default for PrinterConfig {
    fn default() -> Self {
        let indent_style = IndentStyle::default();
        let (indent_text, indent_size) = match indent_style {
            IndentStyle::Spaces(count) => (" ".repeat(count as usize).into(), count as usize),
            IndentStyle::Tabs => ("\t".into(), 4),
        };

        Self { indent_style, indent_text, line_ending: LineEnding::default(), max_width: 100, insert_final_newline: true, trim_trailing_whitespace: true, indent_size }
    }
}

impl PrinterConfig {
    /// Creates a new default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Gets the line ending string
    pub fn line_ending_string(&self) -> &'static str {
        match self.line_ending {
            LineEnding::Unix => "\n",
            LineEnding::Windows => "\r\n",
            LineEnding::Auto => {
                #[cfg(windows)]
                return "\r\n";
                #[cfg(not(windows))]
                return "\n";
            }
        }
    }

    /// Sets the indent style
    pub fn with_indent_style(mut self, style: IndentStyle) -> Self {
        self.indent_style = style;
        let (indent_text, indent_size) = match style {
            IndentStyle::Spaces(count) => (" ".repeat(count as usize).into(), count as usize),
            IndentStyle::Tabs => ("\t".into(), 4),
        };
        self.indent_text = indent_text;
        self.indent_size = indent_size;
        self
    }

    /// Sets the line ending
    pub fn with_line_ending(mut self, ending: LineEnding) -> Self {
        self.line_ending = ending;
        self
    }

    /// Sets the maximum line length
    pub fn with_max_width(mut self, length: usize) -> Self {
        self.max_width = length;
        self
    }
}

/// Responsible for rendering a Document into a string
pub struct Printer {
    config: PrinterConfig,
    output: String,
    indent_level: usize,
    column: usize,
}

impl Printer {
    /// Creates a new printer with the given configuration
    pub fn new(config: PrinterConfig) -> Self {
        Self { config, output: String::new(), indent_level: 0, column: 0 }
    }

    /// Prints the document to a string
    pub fn print(mut self, doc: &Document<'_>) -> String {
        self.render(doc, false);
        self.finalize();
        self.output
    }

    fn finalize(&mut self) {
        if self.config.trim_trailing_whitespace {
            self.output = self.output.trim_end_matches([' ', '\t']).to_string()
        }
        if self.config.insert_final_newline && !self.output.is_empty() && !self.output.ends_with('\n') {
            self.output.push_str(self.config.line_ending_string())
        }
    }

    fn render(&mut self, doc: &Document<'_>, is_broken: bool) {
        match doc {
            Document::Nil => {}
            Document::Text(s) => {
                self.output.push_str(s);
                self.column += s.len()
            }
            Document::Concat(docs) => {
                for d in docs {
                    self.render(d, is_broken)
                }
            }
            Document::Group(d) => {
                let should_break = self.will_break(d);
                self.render(d, should_break)
            }
            Document::Indent(d) => {
                self.indent_level += 1;
                self.render(d, is_broken);
                self.indent_level -= 1
            }
            Document::Line => {
                if is_broken {
                    self.newline()
                }
                else {
                    self.output.push(' ');
                    self.column += 1
                }
            }
            Document::SoftLine => {
                if is_broken {
                    self.newline()
                }
            }
            Document::SoftLineSpace => {
                if is_broken {
                    self.newline()
                }
                else {
                    self.output.push(' ');
                    self.column += 1
                }
            }
            Document::HardLine => self.newline(),
        }
    }

    fn newline(&mut self) {
        if self.config.trim_trailing_whitespace {
            while self.output.ends_with(' ') || self.output.ends_with('\t') {
                let _ = self.output.pop();
            }
        }
        self.output.push_str(self.config.line_ending_string());
        self.write_indent();
        self.column = self.indent_level * self.config.indent_size
    }

    fn write_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str(&self.config.indent_text)
        }
    }

    /// Simple width prediction logic
    fn will_break(&self, doc: &Document<'_>) -> bool {
        let mut width = self.column;
        self.check_width(doc, &mut width)
    }

    fn check_width(&self, doc: &Document<'_>, width: &mut usize) -> bool {
        if *width > self.config.max_width {
            return true;
        }

        match doc {
            Document::Nil => false,
            Document::Text(s) => {
                *width += s.len();
                *width > self.config.max_width
            }
            Document::Concat(docs) => {
                for d in docs {
                    if self.check_width(d, width) {
                        return true;
                    }
                }
                false
            }
            Document::Group(d) => self.check_width(d, width),
            Document::Indent(d) => self.check_width(d, width),
            Document::Line => {
                *width += 1;
                *width > self.config.max_width
            }
            Document::SoftLine => false,
            Document::SoftLineSpace => {
                *width += 1;
                *width > self.config.max_width
            }
            Document::HardLine => true,
        }
    }
}
