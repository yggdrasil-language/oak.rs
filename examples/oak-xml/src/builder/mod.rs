use crate::{
    XmlElementType, XmlLanguage,
    ast::{XmlAttribute, XmlElement, XmlRoot, XmlValue},
    lexer::token_type::XmlTokenType,
};
use core::range::Range;
use oak_core::{
    Builder, BuilderCache, GreenNode, GreenTree, OakDiagnostics, OakError, Parser, Source,
    builder::BuildOutput,
    source::{SourceText, TextEdit},
};

/// XML AST builder.
pub struct XmlBuilder;

fn push_text_child(children: &mut Vec<XmlValue>, text: String) {
    if text.is_empty() {
        return;
    }
    if let Some(XmlValue::Text(last)) = children.last_mut() {
        last.push_str(&text);
        return;
    }
    children.push(XmlValue::Text(text));
}

impl XmlBuilder {
    /// Creates a new `XmlBuilder`.
    pub fn new() -> Self {
        Self
    }

    fn build_root<'a>(&self, green_tree: &GreenNode<'a, XmlLanguage>, source: &SourceText) -> Result<XmlRoot, OakError> {
        let mut children = Vec::new();
        let mut current_offset = 0;

        for child in green_tree.children {
            match child {
                GreenTree::Node(node) => {
                    match node.kind {
                        XmlElementType::Prolog => {
                            // For now we might ignore prolog or handle it if needed
                        }
                        XmlElementType::Element => {
                            children.push(self.build_element(node, current_offset, source)?);
                        }
                        _ => {}
                    }
                    current_offset += node.byte_length as usize;
                }
                GreenTree::Leaf(leaf) => {
                    current_offset += leaf.length as usize;
                }
            }
        }

        let value = if children.len() == 1 { XmlValue::Element(children.remove(0)) } else { children.into_iter().next().map(XmlValue::Element).unwrap_or(XmlValue::Text(String::new())) };

        Ok(XmlRoot { value })
    }

    fn build_element<'a>(&self, node: &GreenNode<'a, XmlLanguage>, offset: usize, source: &SourceText) -> Result<XmlElement, OakError> {
        let mut name = String::new();
        let mut attributes = Vec::new();
        let mut children = Vec::new();
        let mut current_offset = offset;

        for child in node.children {
            match child {
                GreenTree::Node(n) => {
                    match n.kind {
                        XmlElementType::StartTag | XmlElementType::SelfClosingTag => {
                            let mut sub_offset = current_offset;
                            for sub_child in n.children {
                                match sub_child {
                                    GreenTree::Leaf(t) if t.kind == XmlTokenType::Identifier => {
                                        if name.is_empty() {
                                            name = source
                                                .get_text_in(Range {
                                                    start: sub_offset,
                                                    end: sub_offset + t.length as usize,
                                                })
                                                .to_string();
                                        }
                                    }
                                    GreenTree::Node(attr_node) if attr_node.kind == XmlElementType::Attribute => {
                                        attributes.push(self.build_attribute(attr_node, sub_offset, source)?);
                                    }
                                    _ => {}
                                }
                                sub_offset += sub_child.len() as usize;
                            }
                        }
                        XmlElementType::Element => {
                            children.push(XmlValue::Element(self.build_element(n, current_offset, source)?));
                        }
                        _ => {}
                    }
                    current_offset += n.byte_length as usize;
                }
                GreenTree::Leaf(t) => {
                    let text = source
                        .get_text_in(Range {
                            start: current_offset,
                            end: current_offset + t.length as usize,
                        })
                        .to_string();
                    match t.kind {
                        XmlTokenType::Text | XmlTokenType::Identifier => {
                            push_text_child(&mut children, text);
                        }
                        XmlTokenType::Whitespace | XmlTokenType::Newline => {
                            push_text_child(&mut children, text);
                        }
                        _ => {}
                    }
                    current_offset += t.length as usize;
                }
            }
        }

        Ok(XmlElement { name, attributes, children, span: Range { start: offset, end: offset + node.byte_length as usize } })
    }

    fn build_attribute<'a>(&self, node: &GreenNode<'a, XmlLanguage>, offset: usize, source: &SourceText) -> Result<XmlAttribute, OakError> {
        let mut name = String::new();
        let mut value = String::new();
        let mut current_offset = offset;

        for child in node.children {
            match child {
                GreenTree::Leaf(t) => {
                    match t.kind {
                        XmlTokenType::Identifier => {
                            name = source.get_text_in(Range { start: current_offset, end: current_offset + t.length as usize }).to_string();
                        }
                        XmlTokenType::StringLiteral => {
                            let raw = source.get_text_in(Range { start: current_offset, end: current_offset + t.length as usize });
                            // Strip quotes
                            if raw.len() >= 2 {
                                value = raw[1..raw.len() - 1].to_string();
                            }
                            else {
                                value = raw.to_string();
                            }
                        }
                        _ => {}
                    }
                    current_offset += t.length as usize;
                }
                GreenTree::Node(n) => {
                    current_offset += n.byte_length as usize;
                }
            }
        }

        Ok(XmlAttribute { name, value, span: Range { start: offset, end: offset + node.byte_length as usize } })
    }
}

impl Builder<XmlLanguage> for XmlBuilder {
    fn build<'a, S: Source + ?Sized>(&self, text: &S, edits: &[TextEdit], cache: &'a mut impl BuilderCache<XmlLanguage>) -> BuildOutput<XmlLanguage> {
        let source = SourceText::new(text.get_text_in(Range { start: 0, end: text.length() }).to_string());
        let config = XmlLanguage::default();
        let parser = crate::parser::XmlParser::new(&config);
        let parse_output = parser.parse(text, edits, cache);

        let mut diagnostics = Vec::new();
        for error in parse_output.diagnostics {
            diagnostics.push(error);
        }

        let result = parse_output.result.and_then(|green_tree| self.build_root(green_tree, &source));

        OakDiagnostics { result, diagnostics }
    }
}
