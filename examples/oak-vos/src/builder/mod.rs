use crate::{
    ast::{VosDeclaration, VosDeclarationKind, VosField, VosFieldAttribute, VosParameter, VosRoot, VosSyntaxElement, VosSyntaxNode, VosSyntaxSlice, VosSyntaxToken, VosTypeArgument, VosTypeSyntax},
    language::VosLanguage,
    lexer::VosLexer,
    parser::{VosElementType, VosParser},
};
use oak_core::{Builder, BuilderCache, GreenNode, GreenTree, Lexer, OakDiagnostics, OakError, Parser, SourceText, TextEdit, TokenType, parser::session::ParseSession, source::Source};

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
        let mut signature = None;
        let mut parameters = Vec::new();
        let mut body = None;
        let mut return_type = None;
        let mut return_type_expr = None;
        let mut child_offset = offset;
        for child in node.children {
            if let GreenTree::Node(child_node) = child {
                match child_node.kind {
                    VosElementType::Parentheses => {
                        signature = Some(self.slice(child_node, child_offset, source));
                        parameters = self.build_parameters(child_node, child_offset, source)?;
                    }
                    VosElementType::Block => {
                        body = Some(self.slice(child_node, child_offset, source));
                    }
                    VosElementType::ReturnType => {
                        return_type = Some(self.slice(child_node, child_offset, source));
                        return_type_expr = Some(self.build_type_expr(child_node, child_offset, source)?);
                    }
                    _ => {}
                }
            }
            child_offset += match child {
                GreenTree::Node(node) => node.byte_length as usize,
                GreenTree::Leaf(leaf) => leaf.length as usize,
            };
        }
        let mut fields = Vec::new();
        if matches!(kind, VosDeclarationKind::Table | VosDeclarationKind::Class) {
            self.collect_fields(node, offset, source, &mut fields)?;
        }
        Ok(VosDeclaration { kind, name, path, signature, parameters, body, return_type, return_type_expr, fields, span })
    }

    fn build_parameters(&self, node: &GreenNode<'_, VosLanguage>, offset: usize, source: &SourceText) -> Result<Vec<VosParameter>, OakError> {
        let mut tokens = Vec::new();
        self.collect_type_tokens(node, offset, source, &mut tokens);
        let mut index = 0;
        let mut parameters = Vec::new();
        while index < tokens.len() {
            if tokens[index].0 == crate::lexer::VosTokenType::LeftParen {
                index += 1;
                continue;
            }
            if tokens[index].0 == crate::lexer::VosTokenType::RightParen { break; }
            if !matches!(tokens[index].0, crate::lexer::VosTokenType::Identifier) {
                index += 1;
                continue;
            }
            let name_span = tokens[index].1.clone();
            let name = self.token_text(&name_span, source).into_owned();
            index += 1;
            while index < tokens.len() && tokens[index].0.is_ignored() { index += 1; }
            if tokens.get(index).is_none_or(|token| token.0 != crate::lexer::VosTokenType::Colon) { break; }
            index += 1;
            while index < tokens.len() && tokens[index].0.is_ignored() { index += 1; }
            let type_start = tokens.get(index).map(|token| token.1.start).unwrap_or(name_span.end);
            let type_expr = self.parse_type_tokens(&tokens, &mut index, source)?;
            let type_end = self.type_span(&type_expr).end;
            parameters.push(VosParameter {
                name,
                name_span: name_span.clone().into(),
                type_syntax: self.raw_slice(type_start, type_end, source),
                type_expr,
                span: (name_span.start..type_end).into(),
            });
            while index < tokens.len() && tokens[index].0 != crate::lexer::VosTokenType::Comma && tokens[index].0 != crate::lexer::VosTokenType::RightParen { index += 1; }
            if tokens.get(index).is_some_and(|token| token.0 == crate::lexer::VosTokenType::Comma) { index += 1; }
        }
        Ok(parameters)
    }

    fn collect_type_tokens(&self, node: &GreenNode<'_, VosLanguage>, offset: usize, source: &SourceText, tokens: &mut Vec<(crate::lexer::VosTokenType, core::range::Range<usize>)>) {
        let mut child_offset = offset;
        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => tokens.push((leaf.kind, (child_offset..child_offset + leaf.length as usize).into())),
                GreenTree::Node(inner) => self.collect_type_tokens(inner, child_offset, source, tokens),
            }
            child_offset += match child { GreenTree::Node(inner) => inner.byte_length as usize, GreenTree::Leaf(leaf) => leaf.length as usize };
        }
        let _ = source;
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
        let type_expr = self.build_type_expr(node, offset, source)?;
        Ok(VosField { name, name_span, attributes, type_syntax, type_expr, default_value, span: span.into() })
    }

    fn build_type_expr(&self, field: &GreenNode<'_, VosLanguage>, field_offset: usize, source: &SourceText) -> Result<VosTypeSyntax, OakError> {
        let mut child_offset = field_offset;
        for child in field.children {
            if let GreenTree::Node(node) = child {
                if node.kind == VosElementType::TypeSyntax {
                    let mut tokens = Vec::new();
                    let mut token_offset = child_offset;
                    for type_child in node.children {
                        if let GreenTree::Leaf(leaf) = type_child {
                            if !matches!(leaf.kind, crate::lexer::VosTokenType::Whitespace | crate::lexer::VosTokenType::Comment) {
                                tokens.push((leaf.kind, (token_offset..token_offset + leaf.length as usize).into()));
                            }
                        }
                        token_offset += match type_child {
                            GreenTree::Node(inner) => inner.byte_length as usize,
                            GreenTree::Leaf(leaf) => leaf.length as usize,
                        };
                    }
                    let mut index = 0;
                    let type_expr = self.parse_type_tokens(&tokens, &mut index, source)?;
                    if let Some((_, span)) = tokens.get(index) {
                        return Err(OakError::expected_token("end of VOS type", span.start, None));
                    }
                    return Ok(type_expr);
                }
            }
            child_offset += match child {
                GreenTree::Node(node) => node.byte_length as usize,
                GreenTree::Leaf(leaf) => leaf.length as usize,
            };
        }
        Err(OakError::expected_token("VOS type syntax", field_offset, None))
    }

    fn parse_type_tokens(&self, tokens: &[(crate::lexer::VosTokenType, core::range::Range<usize>)], index: &mut usize, source: &SourceText) -> Result<VosTypeSyntax, OakError> {
        let Some((kind, span)) = tokens.get(*index) else {
            return Err(OakError::expected_token("VOS type syntax", 0, None));
        };
        let start = span.start;
        let mut value = if *kind == crate::lexer::VosTokenType::Operator && self.token_text(span, source) == "&" {
            *index += 1;
            let target = self.parse_type_tokens(tokens, index, source)?;
            let (target, target_optional) = match target {
                VosTypeSyntax::Optional { inner, span } => (*inner, Some(span.end)),
                target => (target, None),
            };
            let end = self.type_span(&target).end;
            let reference = VosTypeSyntax::Reference { target: Box::new(target), span: (start..end).into() };
            if let Some(optional_end) = target_optional {
                VosTypeSyntax::Optional { inner: Box::new(reference), span: (start..optional_end).into() }
            } else {
                reference
            }
        } else if *kind == crate::lexer::VosTokenType::LeftBracket {
            *index += 1;
            let element = self.parse_type_tokens(tokens, index, source)?;
            let close = tokens.get(*index).ok_or_else(|| OakError::unexpected_eof(start, None))?;
            if close.0 != crate::lexer::VosTokenType::RightBracket {
                return Err(OakError::expected_token("]", close.1.start, None));
            }
            *index += 1;
            VosTypeSyntax::List { element: Box::new(element), span: (start..close.1.end).into() }
        } else if *kind == crate::lexer::VosTokenType::Identifier {
            let mut path = vec![self.token_text(span, source).into_owned()];
            let mut end = span.end;
            *index += 1;
            while *index + 1 < tokens.len() && tokens[*index].0 == crate::lexer::VosTokenType::Colon && tokens[*index + 1].0 == crate::lexer::VosTokenType::Colon {
                *index += 2;
                let segment = tokens.get(*index).ok_or_else(|| OakError::unexpected_eof(end, None))?;
                if segment.0 != crate::lexer::VosTokenType::Identifier {
                    return Err(OakError::expected_token("type path segment", segment.1.start, None));
                }
                path.push(self.token_text(&segment.1, source).into_owned());
                end = segment.1.end;
                *index += 1;
            }
            if tokens.get(*index).is_some_and(|token| token.0 == crate::lexer::VosTokenType::Less) {
                *index += 1;
                let mut arguments = Vec::new();
                loop {
                    let argument = tokens.get(*index).ok_or_else(|| OakError::unexpected_eof(end, None))?;
                    if argument.0 == crate::lexer::VosTokenType::Greater {
                        return Err(OakError::expected_token("generic argument", argument.1.start, None));
                    }
                    let parsed = if matches!(argument.0, crate::lexer::VosTokenType::NumberLiteral | crate::lexer::VosTokenType::StringLiteral | crate::lexer::VosTokenType::BooleanLiteral | crate::lexer::VosTokenType::NullLiteral) {
                        *index += 1;
                        VosTypeArgument::Literal(self.raw_slice(argument.1.start, argument.1.end, source))
                    } else {
                        VosTypeArgument::Type(self.parse_type_tokens(tokens, index, source)?)
                    };
                    end = match &parsed {
                        VosTypeArgument::Type(value) => self.type_span(value).end,
                        VosTypeArgument::Literal(value) => value.span.end,
                    };
                    arguments.push(parsed);
                    let separator = tokens.get(*index).ok_or_else(|| OakError::unexpected_eof(end, None))?;
                    if separator.0 == crate::lexer::VosTokenType::Greater {
                        end = separator.1.end;
                        *index += 1;
                        break;
                    }
                    if separator.0 != crate::lexer::VosTokenType::Comma {
                        return Err(OakError::expected_token(", or >", separator.1.start, None));
                    }
                    *index += 1;
                }
                VosTypeSyntax::Generic { path, arguments, span: (start..end).into() }
            } else {
                VosTypeSyntax::Named { path, span: (start..end).into() }
            }
        } else {
            return Err(OakError::expected_token("VOS type syntax", span.start, None));
        };
        if tokens.get(*index).is_some_and(|token| token.0 == crate::lexer::VosTokenType::Question) {
            let question = &tokens[*index].1;
            *index += 1;
            value = VosTypeSyntax::Optional { inner: Box::new(value), span: (start..question.end).into() };
        }
        Ok(value)
    }

    fn token_text<'a>(&self, span: &core::range::Range<usize>, source: &'a SourceText) -> std::borrow::Cow<'a, str> {
        source.get_text_in(span.clone().into())
    }

    fn type_span<'a>(&self, value: &'a VosTypeSyntax) -> &'a core::range::Range<usize> {
        match value {
            VosTypeSyntax::Named { span, .. } | VosTypeSyntax::Reference { span, .. } | VosTypeSyntax::Optional { span, .. } | VosTypeSyntax::List { span, .. } | VosTypeSyntax::Generic { span, .. } => span,
        }
    }

    fn raw_slice(&self, start: usize, end: usize, source: &SourceText) -> VosSyntaxSlice {
        let span: core::range::Range<usize> = (start..end).into();
        let text = source.get_text_in(span.clone()).into_owned();
        VosSyntaxSlice { text, span }
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
