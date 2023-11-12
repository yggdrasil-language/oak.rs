use crate::{
    ast::{VosDeclaration, VosDeclarationKind, VosRoot, VosSyntaxElement, VosSyntaxNode, VosSyntaxToken},
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
        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    if leaf.kind == crate::lexer::VosTokenType::Identifier && name.is_none() {
                        let text = source.get_text_in((current_offset..current_offset + leaf.length as usize).into());
                        name = Some(text.into_owned());
                    }
                    current_offset += leaf.length as usize;
                }
                GreenTree::Node(child_node) => {
                    current_offset += child_node.byte_length as usize;
                }
            }
        }
        Ok(VosDeclaration { kind, name, span })
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
