#![doc = include_str!("readme.md")]

use crate::{XmlElementType, XmlLanguage, XmlTokenType};
use core::range::Range;
use oak_core::{source::Source, tree::RedNode};
use std::borrow::Cow;

/// Root node of the XML AST.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct XmlRoot {
    /// The root value.
    pub value: XmlValue,
}

/// A node in the XML red tree.
pub type XmlNode<'a> = RedNode<'a, XmlLanguage>;

/// Represents a value in XML.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub enum XmlValue {
    /// An XML element.
    Element(XmlElement),
    /// Text content.
    Text(String),
    /// A comment.
    Comment(String),
    /// CDATA section.
    CData(String),
    /// Processing instruction.
    ProcessingInstruction(XmlPI),
    /// A fragment of multiple values.
    Fragment(Vec<XmlValue>),
}

/// Represents an XML element.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct XmlElement {
    /// The tag name.
    pub name: String,
    /// Attributes of the element.
    pub attributes: Vec<XmlAttribute>,
    /// Children of the element.
    pub children: Vec<XmlValue>,
    /// Source range of the element.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an XML attribute.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct XmlAttribute {
    /// The attribute name.
    pub name: String,
    /// The attribute value.
    pub value: String,
    /// Source range of the attribute.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

/// Represents an XML processing instruction.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone, Debug, PartialEq)]
pub struct XmlPI {
    /// The PI target.
    pub target: String,
    /// The PI data.
    pub data: Option<String>,
    /// Source range of the PI.
    #[cfg_attr(feature = "serde", serde(with = "oak_core::serde_range"))]
    pub span: Range<usize>,
}

impl XmlValue {
    /// Returns the element if the value is an element.
    pub fn as_element(&self) -> Option<&XmlElement> {
        match self {
            XmlValue::Element(e) => Some(e),
            _ => None,
        }
    }

    /// Returns the text if the value is text.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            XmlValue::Text(s) => Some(s),
            _ => None,
        }
    }

    /// Converts the value to an XML string representation.
    pub fn to_string(&self) -> String {
        match self {
            XmlValue::Text(t) => t.clone(),
            XmlValue::Comment(c) => format!("<!--{}-->", c),
            XmlValue::CData(d) => format!("<![CDATA[{}]]>", d),
            XmlValue::ProcessingInstruction(pi) => {
                if let Some(ref data) = pi.data {
                    format!("<?{} {}?>", pi.target, data)
                }
                else {
                    format!("<?{}?>", pi.target)
                }
            }
            XmlValue::Fragment(fs) => {
                let mut s = String::new();
                for f in fs {
                    s.push_str(&f.to_string());
                }
                s
            }
            XmlValue::Element(e) => {
                let mut s = format!("<{}", e.name);
                for attr in &e.attributes {
                    s.push_str(&format!(" {}=\"{}\"", attr.name, attr.value));
                }
                if e.children.is_empty() {
                    s.push_str("/>");
                }
                else {
                    s.push('>');
                    for child in &e.children {
                        s.push_str(&child.to_string());
                    }
                    s.push_str(&format!("</{}>", e.name));
                }
                s
            }
        }
    }
}

/// Extension trait for XML red nodes.
pub trait XmlNodeExt<'a> {
    /// Returns the tag name of the element.
    fn tag_name<'s, S: Source + ?Sized>(&self, source: &'s S) -> Option<Cow<'s, str>>;
    /// Returns the attributes of the element.
    fn attributes<S: Source + ?Sized>(&self, source: &S) -> Vec<(String, String)>;
    /// Returns an iterator over the element's children that are elements.
    fn xml_children(&self) -> impl Iterator<Item = RedNode<'a, XmlLanguage>>;
    /// Returns a recursive iterator over all element descendants.
    fn xml_children_recursive(&self) -> impl Iterator<Item = RedNode<'a, XmlLanguage>>;
    /// Returns the text content of the node.
    fn text<S: Source + ?Sized>(&self, source: &S) -> String;
    /// Reads an attribute value by name.
    fn read_attr<S: Source + ?Sized>(&self, source: &S, name: &str) -> Option<String>;
}

impl<'a> XmlNodeExt<'a> for RedNode<'a, XmlLanguage> {
    fn tag_name<'s, S: Source + ?Sized>(&self, source: &'s S) -> Option<Cow<'s, str>> {
        if self.green.kind != XmlElementType::Element {
            return None;
        }
        for child in self.children() {
            if let Some(node) = child.as_node() {
                if node.green.kind == XmlElementType::StartTag || node.green.kind == XmlElementType::SelfClosingTag {
                    for gc in node.children() {
                        if let Some(leaf) = gc.as_token() {
                            if leaf.kind == XmlTokenType::Identifier {
                                return Some(source.get_text_in(leaf.span));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    fn attributes<S: Source + ?Sized>(&self, source: &S) -> Vec<(String, String)> {
        let mut attrs = Vec::new();
        if self.green.kind != XmlElementType::Element {
            return attrs;
        }
        for child in self.children() {
            if let Some(node) = child.as_node() {
                if node.green.kind == XmlElementType::StartTag || node.green.kind == XmlElementType::SelfClosingTag {
                    for gc in node.children() {
                        if let Some(n) = gc.as_node() {
                            if n.green.kind == XmlElementType::Attribute {
                                let mut name = String::new();
                                let mut value = String::new();
                                for ggc in n.children() {
                                    if let Some(leaf) = ggc.as_token() {
                                        if leaf.kind == XmlTokenType::Identifier {
                                            name = source.get_text_in(leaf.span).into_owned();
                                        }
                                        else if leaf.kind == XmlTokenType::AttributeValue
                                            || leaf.kind == XmlTokenType::StringLiteral
                                        {
                                            let v = source.get_text_in(leaf.span);
                                            value = v.trim_matches('"').trim_matches('\'').to_string();
                                        }
                                    }
                                }
                                if !name.is_empty() {
                                    attrs.push((name, value));
                                }
                            }
                        }
                    }
                }
            }
        }
        attrs
    }

    fn xml_children(&self) -> impl Iterator<Item = RedNode<'a, XmlLanguage>> {
        self.children().filter_map(|c| c.as_node().filter(|node| node.green.kind == XmlElementType::Element))
    }

    fn xml_children_recursive(&self) -> impl Iterator<Item = RedNode<'a, XmlLanguage>> {
        let mut stack = Vec::new();
        for child in self.xml_children() {
            stack.push(child);
        }

        std::iter::from_fn(move || {
            let next = stack.pop()?;
            let children = next.xml_children().collect::<Vec<_>>();
            for child in children.into_iter().rev() {
                stack.push(child);
            }
            Some(next)
        })
    }

    fn text<S: Source + ?Sized>(&self, source: &S) -> String {
        let mut text = String::new();
        for child in self.children() {
            if let Some(leaf) = child.as_token() {
                if leaf.kind == XmlTokenType::Text {
                    text.push_str(&source.get_text_in(leaf.span));
                }
            }
            else if let Some(node) = child.as_node() {
                if node.green.kind == XmlElementType::Element {
                    text.push_str(&node.text(source));
                }
            }
        }
        text
    }

    fn read_attr<S: Source + ?Sized>(&self, source: &S, name: &str) -> Option<String> {
        self.attributes(source).into_iter().find(|(n, _)| n == name).map(|(_, v)| v)
    }
}
