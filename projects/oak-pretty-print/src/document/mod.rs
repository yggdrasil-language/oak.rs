use alloc::{borrow::Cow, boxed::Box, string::String, vec::Vec};
use core::fmt;

/// Document printer implementation
pub mod printer;

pub use self::printer::{IndentStyle, LineEnding, Printer, PrinterConfig};

/// Document abstraction for describing layout logic
#[derive(Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize))]
pub enum Document<'a> {
    /// Empty document
    Nil,
    /// Plain text
    Text(Cow<'a, str>),
    /// Concatenation of multiple documents
    Concat(Vec<Document<'a>>),
    /// A group of documents, treated as a single unit for line break calculations
    Group(Box<Document<'a>>),
    /// Increase indentation level
    Indent(Box<Document<'a>>),
    /// Force a line break
    Line,
    /// Soft line break: a line break if the group breaks, otherwise empty
    SoftLine,
    /// Soft line break with space: a line break if the group breaks, otherwise a space
    SoftLineSpace,
    /// Force a line break and propagate it to parent groups
    HardLine,
}

impl<'a> fmt::Debug for Document<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        #[cfg(debug_assertions)]
        {
            match self {
                Document::Nil => write!(f, "Nil"),
                Document::Text(s) => write!(f, "Text({:?})", s),
                Document::Concat(docs) => f.debug_list().entries(docs).finish(),
                Document::Group(d) => f.debug_tuple("Group").field(d).finish(),
                Document::Indent(d) => f.debug_tuple("Indent").field(d).finish(),
                Document::Line => write!(f, "Line"),
                Document::SoftLine => write!(f, "SoftLine"),
                Document::SoftLineSpace => write!(f, "SoftLineSpace"),
                Document::HardLine => write!(f, "HardLine"),
            }
        }
        #[cfg(not(debug_assertions))]
        {
            match self {
                Document::Nil => write!(f, "Doc::Nil"),
                Document::Text(_) => write!(f, "Doc::Text"),
                Document::Concat(_) => write!(f, "Doc::Concat"),
                Document::Group(_) => write!(f, "Doc::Group"),
                Document::Indent(_) => write!(f, "Doc::Indent"),
                Document::Line => write!(f, "Doc::Line"),
                Document::SoftLine => write!(f, "Doc::SoftLine"),
                Document::SoftLineSpace => write!(f, "Doc::SoftLineSpace"),
                Document::HardLine => write!(f, "Doc::HardLine"),
            }
        }
    }
}

impl<'a> Document<'a> {
    /// Renders the document into a string using default configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oak_pretty_print::Document;
    /// let doc = Document::concat(vec![
    ///     Document::text("hello"),
    ///     Document::HardLine,
    ///     Document::text("world"),
    /// ]);
    /// let output = doc.render();
    /// assert_eq!(output, "hello\nworld");
    /// ```
    pub fn render(&self) -> String {
        let config = self::printer::PrinterConfig::default();
        self::printer::Printer::new(config).print(self)
    }

    /// Renders the document into a string using the provided configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use oak_pretty_print::{Document, document::printer::PrinterConfig};
    /// let doc = Document::concat(vec![
    ///     Document::text("hello"),
    ///     Document::HardLine,
    ///     Document::text("world"),
    /// ]);
    /// let config = PrinterConfig::default();
    /// let output = doc.render_with_config(config);
    /// assert_eq!(output, "hello\nworld");
    /// ```
    pub fn render_with_config(&self, config: self::printer::PrinterConfig) -> String {
        self::printer::Printer::new(config).print(self)
    }

    /// Creates a text document
    pub fn text<S: Into<Cow<'a, str>>>(text: S) -> Self {
        Document::Text(text.into())
    }

    /// Concatenates multiple documents
    pub fn concat(docs: Vec<Document<'a>>) -> Self {
        Document::Concat(docs)
    }

    /// Creates a grouped document
    pub fn group(doc: Document<'a>) -> Self {
        Document::Group(Box::new(doc))
    }

    /// Creates an indented document
    pub fn indent(doc: Document<'a>) -> Self {
        Document::Indent(Box::new(doc))
    }

    /// Helper method: Joins multiple documents with a specified separator
    pub fn join<I>(docs: I, separator: Document<'a>) -> Self
    where
        I: IntoIterator<Item = Document<'a>>,
    {
        let mut result = Vec::new();
        for (i, doc) in docs.into_iter().enumerate() {
            if i > 0 {
                result.push(separator.clone());
            }
            result.push(doc);
        }
        Document::Concat(result)
    }
}

impl<'a> From<Cow<'a, str>> for Document<'a> {
    fn from(s: Cow<'a, str>) -> Self {
        Document::Text(s)
    }
}

impl From<String> for Document<'_> {
    fn from(s: String) -> Self {
        Document::Text(s.into())
    }
}

impl<'a> From<&'a str> for Document<'a> {
    fn from(s: &'a str) -> Self {
        Document::Text(s.into())
    }
}

/// Extension trait for joining an iterator of documents
pub trait JoinDoc<'a> {
    /// Joins the documents with the given separator
    fn join_doc(self, separator: Document<'a>) -> Document<'a>;
}

impl<'a, I> JoinDoc<'a> for I
where
    I: IntoIterator<Item = Document<'a>>,
{
    fn join_doc(self, separator: Document<'a>) -> Document<'a> {
        Document::join(self, separator)
    }
}
