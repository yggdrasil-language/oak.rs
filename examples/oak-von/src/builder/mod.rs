use crate::{
    VonLanguage,
    ast::{VonArray, VonBoolean, VonEnum, VonField, VonNull, VonNumber, VonObject, VonRoot, VonString, VonValue},
    lexer::{VonLexer, VonTokenType},
    parser::{VonElementType, VonParser},
};
use oak_core::{Builder, BuilderCache, GreenNode, Lexer, OakDiagnostics, OakError, Parser, SourceText, TextEdit, parser::session::ParseSession, source::Source};

/// A builder for VON AST and diagnostic results.
#[derive(Clone)]
pub struct VonBuilder<'config> {
    config: &'config VonLanguage,
}

impl<'config> VonBuilder<'config> {
    /// Creates a new `VonBuilder` with the given configuration.
    pub fn new(config: &'config VonLanguage) -> Self {
        Self { config }
    }
}

impl<'config> Builder<VonLanguage> for VonBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<VonLanguage>) -> OakDiagnostics<VonRoot> {
        let parser = VonParser::new(self.config);
        let lexer = VonLexer::new(self.config);

        let mut parse_session = ParseSession::<VonLanguage>::default();
        lexer.lex(source, edits, &mut parse_session);
        let parse_result = parser.parse(source, edits, &mut parse_session);

        match parse_result.result {
            Ok(green_tree) => {
                let text = source.get_text_in((0..source.length()).into());
                let source_text = SourceText::new(text.into_owned());
                match self.build_root(&green_tree, &source_text) {
                    Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics: parse_result.diagnostics },
                    Err(build_error) => {
                        let mut diagnostics = parse_result.diagnostics;
                        diagnostics.push(build_error.clone());
                        OakDiagnostics { result: Err(build_error), diagnostics }
                    }
                }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl<'config> VonBuilder<'config> {
    fn build_root<'a>(&self, green_tree: &GreenNode<'a, VonLanguage>, source: &SourceText) -> Result<VonRoot, OakError> {
        // eprintln!("GreenTree: {:#?}", green_tree);
        let root_node = green_tree
            .children
            .iter()
            .find_map(|child| match child {
                oak_core::GreenTree::Node(n) => Some(n),
                oak_core::GreenTree::Leaf(_) => None,
            })
            .ok_or_else(|| OakError::unexpected_eof(0, None))?;

        let value = self.build_value(root_node, 0, source)?;
        Ok(VonRoot { value })
    }

    fn build_value<'a>(&self, node: &GreenNode<'a, VonLanguage>, offset: usize, source: &SourceText) -> Result<VonValue, OakError> {
        let span: oak_core::Range<usize> = (offset..offset + node.byte_length as usize).into();

        match node.kind {
            VonElementType::Object => {
                let mut fields = Vec::new();
                let mut current_offset = offset;
                for child in node.children {
                    match child {
                        oak_core::GreenTree::Node(n) => {
                            if n.kind == VonElementType::ObjectEntry {
                                fields.push(self.build_field(n, current_offset, source)?);
                            }
                            current_offset += n.byte_length as usize;
                        }
                        oak_core::GreenTree::Leaf(l) => {
                            current_offset += l.length as usize;
                        }
                    }
                }
                Ok(VonValue::Object(VonObject { fields, span }))
            }
            VonElementType::Array => {
                let mut elements = Vec::new();
                let mut current_offset = offset;
                for child in node.children {
                    match child {
                        oak_core::GreenTree::Node(n) => {
                            if n.kind == VonElementType::ArrayElement {
                                elements.push(self.build_value(n, current_offset, source)?);
                            }
                            current_offset += n.byte_length as usize;
                        }
                        oak_core::GreenTree::Leaf(l) => {
                            current_offset += l.length as usize;
                        }
                    }
                }
                Ok(VonValue::Array(VonArray { elements, span }))
            }
            VonElementType::Enum => {
                let mut variant = None;
                let mut payload = None;
                let mut current_offset = offset;
                for child in node.children {
                    match child {
                        oak_core::GreenTree::Node(n) => {
                            payload = Some(Box::new(self.build_value(n, current_offset, source)?));
                            current_offset += n.byte_length as usize;
                        }
                        oak_core::GreenTree::Leaf(l) => {
                            if l.kind == VonTokenType::Identifier {
                                let v_span: oak_core::Range<usize> = (current_offset..current_offset + l.length as usize).into();
                                variant = Some(source.get_text_in(v_span).to_string());
                            }
                            current_offset += l.length as usize;
                        }
                    }
                }
                let variant = variant.ok_or_else(|| OakError::expected_token("Variant", span.start, None))?;
                Ok(VonValue::Enum(VonEnum { variant, payload, span }))
            }
            VonElementType::Value | VonElementType::ArrayElement | VonElementType::Root => {
                let mut current_offset = offset;
                for child in node.children {
                    if let oak_core::GreenTree::Node(n) = child {
                        return self.build_value(n, current_offset, source);
                    }
                    else if let oak_core::GreenTree::Leaf(l) = child {
                        if l.kind == VonTokenType::StringLiteral {
                            let s_span: oak_core::Range<usize> = (current_offset..current_offset + l.length as usize).into();
                            let mut text = source.get_text_in(s_span.clone());
                            // Handle raw strings raw"..." or raw'...'
                            let is_raw = text.starts_with("raw") && (text[3..].starts_with('"') || text[3..].starts_with('\''));
                            if is_raw {
                                match text {
                                    std::borrow::Cow::Borrowed(s) => text = std::borrow::Cow::Borrowed(&s[3..]),
                                    std::borrow::Cow::Owned(ref mut s) => {
                                        s.drain(..3);
                                    }
                                }
                            }
                            // Handle symmetric quotes: find the number of consecutive quotes at the beginning
                            let quote_char = text.chars().next().unwrap();
                            let mut quote_count = 0;
                            for c in text.chars() {
                                if c == quote_char {
                                    quote_count += 1;
                                }
                                else {
                                    break;
                                }
                            }
                            let value = if text.len() >= quote_count * 2 { &text[quote_count..text.len() - quote_count] } else { "" };
                            return Ok(VonValue::String(VonString { value: value.to_string(), span: s_span }));
                        }
                        else if l.kind == VonTokenType::NumberLiteral {
                            let n_span: oak_core::Range<usize> = (current_offset..current_offset + l.length as usize).into();
                            let text = source.get_text_in(n_span.clone());
                            // eprintln!("Parsing number: '{}' at span {:?}", text, n_span);
                            let clean_text = text.replace('_', "");
                            let value = clean_text.parse::<f64>().map_err(|e| {
                                OakError::custom_error(format!(
                                    "Numeric parse failure: text='{}', error={:?}, span={:?}, source_around='{}'",
                                    text,
                                    e,
                                    n_span,
                                    source.get_text_in(oak_core::Range::from(n_span.start.saturating_sub(10)..n_span.end.saturating_add(10)))
                                ))
                            })?;
                            return Ok(VonValue::Number(VonNumber { value, span: n_span }));
                        }
                        else if l.kind == VonTokenType::BoolLiteral {
                            let b_span: oak_core::Range<usize> = (current_offset..current_offset + l.length as usize).into();
                            let text = source.get_text_in(b_span.clone());
                            let value = text == "true";
                            return Ok(VonValue::Boolean(VonBoolean { value, span: b_span }));
                        }
                        else if l.kind == VonTokenType::NullLiteral {
                            let n_span: oak_core::Range<usize> = (current_offset..current_offset + l.length as usize).into();
                            return Ok(VonValue::Null(VonNull { span: n_span }));
                        }
                        current_offset += l.length as usize;
                    }
                }
                Err(OakError::unexpected_eof(span.start, None))
            }
            _ => Err(OakError::syntax_error(format!("Unexpected node kind: {:?}", node.kind), span.start, None)),
        }
    }

    fn build_field<'a>(&self, node: &GreenNode<'a, VonLanguage>, offset: usize, source: &SourceText) -> Result<VonField, OakError> {
        let span: oak_core::Range<usize> = (offset..offset + node.byte_length as usize).into();

        let mut name = None;
        let mut value = None;
        let mut current_offset = offset;

        for child in node.children {
            match child {
                oak_core::GreenTree::Node(n) => {
                    if n.kind == VonElementType::Value || n.kind == VonElementType::Object || n.kind == VonElementType::Array || n.kind == VonElementType::Enum {
                        value = Some(self.build_value(n, current_offset, source)?);
                    }
                    current_offset += n.byte_length as usize
                }
                oak_core::GreenTree::Leaf(l) => {
                    if l.kind == VonTokenType::Identifier || l.kind == VonTokenType::StringLiteral {
                        let b_span: oak_core::Range<usize> = (current_offset..current_offset + l.length as usize).into();
                        let text = source.get_text_in(b_span.clone());
                        name = Some(text.trim_matches('"').to_string());
                    }
                    current_offset += l.length as usize
                }
            }
        }

        let name = name.ok_or_else(|| OakError::expected_token("Key", span.start, None))?;
        let value = value.ok_or_else(|| OakError::expected_token("Value", span.start, None))?;

        Ok(VonField { name, value, span })
    }
}
