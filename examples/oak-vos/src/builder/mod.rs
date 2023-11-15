use crate::{
    ast::{VosDeclaration, VosDeclarationKind, VosField, VosFieldAttribute, VosRoot, VosSyntaxElement, VosSyntaxNode, VosSyntaxSlice, VosSyntaxToken},
    language::VosLanguage,
    lexer::VosLexer,
    parser::{VosElementType, VosParser},
};
use oak_core::{Builder, BuilderCache, GreenNode, GreenTree, Lexer, OakDiagnostics, OakError, Parser, SourceText, TextEdit, parser::session::ParseSession, source::Source};

/// Oak Builder for the initial VOS declaration AST.
#[derive(Clone, Debug, Default)]
pub struct VosBuilder;

impl VosBuilder {
    /// Creates a VOS Builder.
    pub fn new(_config: &VosLanguage) -> Self { Self }

    fn kind(element: VosElementType) -> Option<VosDeclarationKind> {
        Some(match element {
            VosElementType::Namespace => VosDeclarationKind::Namespace,
            VosElementType::Table => VosDeclarationKind::Table,
            VosElementType::Class => VosDeclarationKind::Class,
            VosElementType::Enums => VosDeclarationKind::Enums,
            VosElementType::Flags => VosDeclarationKind::Flags,
            VosElementType::Obsolete => VosDeclarationKind::Obsolete,
            VosElementType::Using => VosDeclarationKind::Using,
            VosElementType::Const => VosDeclarationKind::Const,
            VosElementType::Service => VosDeclarationKind::Service,
            VosElementType::Query => VosDeclarationKind::Query,
            VosElementType::Udf => VosDeclarationKind::Udf,
            VosElementType::Micro => VosDeclarationKind::Micro,
            _ => return None,
        })
    }

    fn build_declaration<'a>(&self, node: &GreenNode<'a, VosLanguage>, offset: usize, source: &SourceText) -> Result<VosDeclaration, OakError> {
        let kind = Self::kind(node.kind).ok_or_else(|| OakError::expected_token("VOS declaration", offset, None))?;
        let span = (offset..offset + node.byte_length as usize).into();
        let mut current_offset = offset;
        let mut name = None;
        let mut path = Vec::new();
        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    if leaf.kind == crate::lexer::VosTokenType::Identifier && name.is_none() {
                        let text = source.get_text_in((current_offset..current_offset + leaf.length as usize).into());
                        name = Some(text.into_owned());
                    }
                    if leaf.kind == crate::lexer::VosTokenType::Identifier {
                        let text = source.get_text_in((current_offset..current_offset + leaf.length as usize).into());
                        path.push(text.into_owned());
                    }
                    current_offset += leaf.length as usize;
                }
                GreenTree::Node(child_node) => {
                    current_offset += child_node.byte_length as usize;
                }
            }
        }
        let path = match kind {
            VosDeclarationKind::Namespace | VosDeclarationKind::Using => Some(path),
            _ => None,
        };
        let mut fields = Vec::new();
        if matches!(kind, VosDeclarationKind::Table | VosDeclarationKind::Class) {
            self.collect_fields(node, offset, source, &mut fields)?;
        }
        Ok(VosDeclaration { kind, name, path, fields, span })
    }

    fn collect_fields<'a>(&self, tree: &GreenNode<'a, VosLanguage>, offset: usize, source: &SourceText, fields: &mut Vec<VosField>) -> Result<(), OakError> {
        let mut child_offset = offset;
        for child in tree.children {
            match child {
                GreenTree::Node(node) if node.kind == VosElementType::Field => {
                    fields.push(self.build_field(node, child_offset, source)?);
                }
                GreenTree::Node(node) => self.collect_fields(node, child_offset, source, fields)?,
                GreenTree::Leaf(_) => {}
            }
            child_offset += match child {
                GreenTree::Node(node) => node.byte_length as usize,
                GreenTree::Leaf(leaf) => leaf.length as usize,
            };
        }
        Ok(())
    }

    fn build_field<'a>(&self, node: &GreenNode<'a, VosLanguage>, offset: usize, source: &SourceText) -> Result<VosField, OakError> {
        let span = offset..offset + node.byte_length as usize;
        let mut child_offset = offset;
        let mut name = None;
        let mut name_span = None;
        let mut attributes = Vec::new();
        let mut type_syntax = None;
        let mut default_value = None;
        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    if leaf.kind == crate::lexer::VosTokenType::Identifier && name.is_none() {
                        let token_span = child_offset..child_offset + leaf.length as usize;
                        name = Some(source.get_text_in(token_span.clone().into()).into_owned());
                        name_span = Some(token_span.into());
                    }
                }
                GreenTree::Node(child_node) => {
                    match child_node.kind {
                        VosElementType::FieldAttribute => attributes.push(self.build_attribute(child_node, child_offset, source)),
                        VosElementType::TypeSyntax => type_syntax = Some(self.slice(child_node, child_offset, source)),
                        VosElementType::DefaultValue => default_value = Some(self.slice(child_node, child_offset, source)),
                        _ => {}
                    }
                }
            }
            child_offset += match child {
                GreenTree::Node(node) => node.byte_length as usize,
                GreenTree::Leaf(leaf) => leaf.length as usize,
            };
        }
        let name = name.ok_or_else(|| OakError::expected_token("VOS field name", offset, None))?;
        let name_span = name_span.expect("field name has a span");
        let type_syntax = type_syntax.ok_or_else(|| OakError::expected_token("VOS field type", offset, None))?;
        Ok(VosField { name, name_span, attributes, type_syntax, default_value, span: span.into() })
    }

    fn build_attribute<'a>(&self, node: &GreenNode<'a, VosLanguage>, offset: usize, source: &SourceText) -> VosFieldAttribute {
        let syntax = self.slice(node, offset, source);
        let mut child_offset = offset;
        let mut name = None;
        for child in node.children {
            if let GreenTree::Node(inner) = child {
                let inner_attribute = self.build_attribute(inner, child_offset, source);
                if name.is_none() { name = inner_attribute.name; }
            }
            if let GreenTree::Leaf(leaf) = child {
                if leaf.kind == crate::lexer::VosTokenType::Identifier && name.is_none() {
                    name = Some(source.get_text_in((child_offset..child_offset + leaf.length as usize).into()).into_owned());
                }
            }
            child_offset += match child {
                GreenTree::Node(node) => node.byte_length as usize,
                GreenTree::Leaf(leaf) => leaf.length as usize,
            };
        }
        VosFieldAttribute { name, text: syntax.text, span: syntax.span }
    }

    fn slice<'a>(&self, node: &GreenNode<'a, VosLanguage>, offset: usize, source: &SourceText) -> VosSyntaxSlice {
        let mut start = None;
        let mut end = offset;
        let mut child_offset = offset;
        for child in node.children {
            match child {
                GreenTree::Node(inner) => {
                    let inner_slice = self.slice(inner, child_offset, source);
                    start.get_or_insert(inner_slice.span.start);
                    end = inner_slice.span.end;
                }
                GreenTree::Leaf(leaf) if !matches!(leaf.kind, crate::lexer::VosTokenType::Whitespace | crate::lexer::VosTokenType::Comment) => {
                    start.get_or_insert(child_offset);
                    end = child_offset + leaf.length as usize;
                }
                _ => {}
            }
            child_offset += match child {
                GreenTree::Node(node) => node.byte_length as usize,
                GreenTree::Leaf(leaf) => leaf.length as usize,
            };
        }
        let span = start.unwrap_or(offset)..end;
        let text = source.get_text_in(span.clone().into()).into_owned();
        VosSyntaxSlice { text, span: span.into() }
    }

    fn build_syntax<'a>(&self, tree: &GreenTree<'a, VosLanguage>, offset: usize, source: &SourceText) -> VosSyntaxElement {
        match tree {
            GreenTree::Node(node) => {
                let mut children = Vec::new();
                let mut child_offset = offset;
                for child in node.children {
                    children.push(self.build_syntax(child, child_offset, source));
                    child_offset += match child {
                        GreenTree::Node(child_node) => child_node.byte_length as usize,
                        GreenTree::Leaf(leaf) => leaf.length as usize,
                    };
                }
                VosSyntaxElement::Node(VosSyntaxNode {
                    kind: node.kind,
                    span: (offset..offset + node.byte_length as usize).into(),
                    children,
                })
            }
            GreenTree::Leaf(leaf) => {
                let end = offset + leaf.length as usize;
                VosSyntaxElement::Token(VosSyntaxToken {
                    kind: leaf.kind,
                    span: (offset..end).into(),
                    text: source.get_text_in((offset..end).into()).into_owned(),
                })
            }
        }
    }
}

impl Builder<VosLanguage> for VosBuilder {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<VosLanguage>) -> OakDiagnostics<VosRoot> {
        let parser = VosParser::default();
        let lexer = VosLexer;
        let mut parse_session = ParseSession::<VosLanguage>::default();
        lexer.lex(source, edits, &mut parse_session);
        let parse_result = parser.parse(source, edits, &mut parse_session);
        match parse_result.result {
            Ok(green_tree) => {
                let text = source.get_text_in((0..source.length()).into()).into_owned();
                let source_text = SourceText::new(text.clone());
                let syntax_children = green_tree
                    .children
                    .iter()
                    .scan(0usize, |offset, child| {
                        let element = self.build_syntax(child, *offset, &source_text);
                        *offset += match child {
                            GreenTree::Node(node) => node.byte_length as usize,
                            GreenTree::Leaf(leaf) => leaf.length as usize,
                        };
                        Some(element)
                    })
                    .collect();
                let syntax = VosSyntaxNode {
                    kind: VosElementType::Root,
                    span: (0..text.len()).into(),
                    children: syntax_children,
                };
                let mut declarations = Vec::new();
                let mut offset = 0usize;
                for child in green_tree.children {
                    match child {
                        GreenTree::Node(node) => {
                            if Self::kind(node.kind).is_some() {
                                match self.build_declaration(node, offset, &source_text) {
                                    Ok(declaration) => declarations.push(declaration),
                                    Err(error) => return OakDiagnostics { result: Err(error), diagnostics: parse_result.diagnostics },
                                }
                            }
                            offset += node.byte_length as usize;
                        }
                        GreenTree::Leaf(leaf) => offset += leaf.length as usize,
                    }
                }
                OakDiagnostics { result: Ok(VosRoot { source: text, syntax, declarations }), diagnostics: parse_result.diagnostics }
            }
            Err(error) => OakDiagnostics { result: Err(error), diagnostics: parse_result.diagnostics },
        }
    }
}
