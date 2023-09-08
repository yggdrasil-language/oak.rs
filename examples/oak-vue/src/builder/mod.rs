use crate::{
    VueElementType, VueLanguage,
    ast::{Attribute, AttributeValue, Directive, DirectiveArgument, DirectiveExpression, Modifier, VueAttribute, VueBlock, VueElement, VueInterpolation, VueNode, VueRoot, VueText},
    lexer::token_type::VueTokenType,
};
use core::range::Range;
use oak_core::{
    Builder, BuilderCache, GreenNode, GreenTree, OakDiagnostics, OakError, Parser, Source,
    builder::BuildOutput,
    source::{SourceText, TextEdit},
};

/// Vue AST builder.
pub struct VueBuilder;

impl VueBuilder {
    /// Creates a new `VueBuilder`.
    pub fn new() -> Self {
        Self
    }

    fn build_root<'a>(&self, green_tree: &GreenNode<'a, VueLanguage>, source: &SourceText) -> Result<VueRoot, OakError> {
        let mut blocks = Vec::new();
        let mut current_offset = 0;

        for child in green_tree.children {
            match child {
                GreenTree::Node(node) => {
                    match node.kind {
                        VueElementType::TemplateElement | VueElementType::Program => {
                            blocks.push(self.build_block(node, current_offset, source)?);
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

        Ok(VueRoot { blocks })
    }

    fn build_block<'a>(&self, node: &GreenNode<'a, VueLanguage>, offset: usize, source: &SourceText) -> Result<VueBlock, OakError> {
        let mut name = Range { start: offset, end: offset };
        let mut attributes = Vec::new();
        let mut children = Vec::new();
        let mut current_offset = offset;
        for child in node.children {
            match child {
                GreenTree::Node(n) => {
                    match n.kind {
                        VueElementType::Tag => {
                            let mut sub_offset = current_offset;
                            for sub_child in n.children {
                                match sub_child {
                                    GreenTree::Leaf(t) if t.kind == VueTokenType::Identifier => {
                                        if name.is_empty() {
                                            name = Range { start: sub_offset, end: sub_offset + t.length as usize };
                                        }
                                    }
                                    GreenTree::Node(attr_node) if attr_node.kind == VueElementType::Attribute || attr_node.kind == VueElementType::Directive => {
                                        attributes.push(self.build_attribute(attr_node, sub_offset, source)?);
                                    }
                                    _ => {}
                                }
                                sub_offset += sub_child.len() as usize;
                            }
                        }
                        VueElementType::Element => {
                            children.push(VueNode::Element(self.build_element(n, current_offset, source)?));
                        }
                        VueElementType::Interpolation => {
                            children.push(VueNode::Interpolation(self.build_interpolation(n, current_offset, source)?));
                        }
                        _ => {}
                    }
                    current_offset += n.byte_length as usize;
                }
                GreenTree::Leaf(t) => {
                    match t.kind {
                        VueTokenType::Text => {
                            let block_name = source.get_text_in(name.clone());
                            if block_name == "template" {
                                let text_content = source.get_text_in(Range { start: current_offset, end: current_offset + t.length as usize });
                                if !text_content.trim().is_empty() {
                                    children.push(VueNode::Text(VueText { span: Range { start: current_offset, end: current_offset + t.length as usize } }));
                                }
                            }
                            else {
                            }
                        }
                        _ => {}
                    }
                    current_offset += t.length as usize;
                }
            }
        }

        if name.is_empty() {
            name = match node.kind {
                VueElementType::TemplateElement => Range { start: offset + 1, end: offset + 9 },
                VueElementType::Program => Range { start: offset + 1, end: offset + 7 },
                _ => name,
            };
        }

        let result = VueBlock { name, attributes, children, span: Range { start: offset, end: offset + node.byte_length as usize } };
        Ok(result)
    }

    fn build_element<'a>(&self, node: &GreenNode<'a, VueLanguage>, offset: usize, source: &SourceText) -> Result<VueElement, OakError> {
        let mut tag_name = Range { start: offset, end: offset };
        let mut attributes = Vec::new();
        let mut children = Vec::new();
        let mut current_offset = offset;

        for child in node.children {
            match child {
                GreenTree::Node(n) => {
                    match n.kind {
                        VueElementType::Tag => {
                            let mut sub_offset = current_offset;
                            for sub_child in n.children {
                                match sub_child {
                                    GreenTree::Leaf(t) if t.kind == VueTokenType::Identifier => {
                                        if tag_name.is_empty() {
                                            tag_name = Range { start: sub_offset, end: sub_offset + t.length as usize };
                                        }
                                    }
                                    GreenTree::Node(attr_node) if attr_node.kind == VueElementType::Attribute || attr_node.kind == VueElementType::Directive => {
                                        attributes.push(self.build_attribute(attr_node, sub_offset, source)?);
                                    }
                                    _ => {}
                                }
                                sub_offset += sub_child.len() as usize;
                            }
                        }
                        VueElementType::Element => {
                            children.push(VueNode::Element(self.build_element(n, current_offset, source)?));
                        }
                        VueElementType::Interpolation => {
                            children.push(VueNode::Interpolation(self.build_interpolation(n, current_offset, source)?));
                        }
                        _ => {}
                    }
                    current_offset += n.byte_length as usize;
                }
                GreenTree::Leaf(t) => {
                    match t.kind {
                        VueTokenType::Text => {
                            let content = source.get_text_in(Range { start: current_offset, end: current_offset + t.length as usize });
                            if !content.trim().is_empty() {
                                children.push(VueNode::Text(VueText { span: Range { start: current_offset, end: current_offset + t.length as usize } }));
                            }
                        }
                        _ => {}
                    }
                    current_offset += t.length as usize;
                }
            }
        }

        let result = VueElement { tag_name, attributes, children, span: Range { start: offset, end: offset + node.byte_length as usize } };
        Ok(result)
    }

    fn build_attribute<'a>(&self, node: &GreenNode<'a, VueLanguage>, offset: usize, _source: &SourceText) -> Result<VueAttribute, OakError> {
        let mut name = Range { start: offset, end: offset };
        let mut value = None;
        let mut current_offset = offset;

        let result = if node.kind == VueElementType::Directive {
            let arg: Option<DirectiveArgument> = None;
            let mut modifiers = Vec::new();

            for child in node.children {
                match child {
                    GreenTree::Leaf(t) => {
                        if Self::is_name_token(t.kind) {
                            if name.is_empty() {
                                name = Range { start: current_offset, end: current_offset + t.length as usize };
                            }
                            else {
                                let range = Range { start: current_offset, end: current_offset + t.length as usize };
                                modifiers.push(Modifier { name: range.clone(), span: range });
                            }
                        }
                        else if t.kind == VueTokenType::StringLiteral {
                            value = Some(Range { start: current_offset + 1, end: current_offset + t.length as usize - 1 });
                        }
                        current_offset += t.length as usize;
                    }
                    GreenTree::Node(n) => {
                        if n.kind == VueElementType::AttributeName && name.is_empty() {
                            name = Self::name_range_from_attr_name_node(n, current_offset);
                        }
                        else if n.kind == VueElementType::AttributeValue {
                            let mut sub_offset = current_offset;
                            for sub_child in n.children {
                                if let GreenTree::Leaf(t) = sub_child {
                                    if t.kind == VueTokenType::StringLiteral {
                                        value = Some(Range { start: sub_offset + 1, end: sub_offset + t.length as usize - 1 });
                                    }
                                }
                                sub_offset += sub_child.len() as usize;
                            }
                        }
                        else if n.kind == VueElementType::Modifier {
                            let range = Range { start: current_offset, end: current_offset + n.byte_length as usize };
                            modifiers.push(Modifier { name: range.clone(), span: range });
                        }
                        current_offset += n.byte_length as usize;
                    }
                }
            }

            let value = value.map(|span| DirectiveExpression { span });

            VueAttribute::Directive(Directive { name, arg, modifiers, value, span: Range { start: offset, end: offset + node.byte_length as usize } })
        }
        else {
            for child in node.children {
                match child {
                    GreenTree::Leaf(t) => {
                        if Self::is_name_token(t.kind) {
                            name = Range { start: current_offset, end: current_offset + t.length as usize };
                        }
                        current_offset += t.length as usize;
                    }
                    GreenTree::Node(n) => {
                        if n.kind == VueElementType::AttributeName && name.is_empty() {
                            name = Self::name_range_from_attr_name_node(n, current_offset);
                        }
                        else if n.kind == VueElementType::AttributeValue {
                            let mut sub_offset = current_offset;
                            for sub_child in n.children {
                                if let GreenTree::Leaf(t) = sub_child {
                                    if t.kind == VueTokenType::StringLiteral {
                                        value = Some(Range { start: sub_offset + 1, end: sub_offset + t.length as usize - 1 });
                                    }
                                }
                                sub_offset += sub_child.len() as usize;
                            }
                        }
                        current_offset += n.byte_length as usize;
                    }
                }
            }
            let value = value.map(|span| AttributeValue { span });
            VueAttribute::Attribute(Attribute { name, value, span: Range { start: offset, end: offset + node.byte_length as usize } })
        };
        Ok(result)
    }

    fn is_name_token(kind: VueTokenType) -> bool {
        use VueTokenType::*;
        matches!(
            kind,
            Identifier
                | For
                | In
                | Of
                | As
                | If
                | Else
                | From
                | Default
                | Const
                | Let
                | Var
                | Function
                | Return
                | Break
                | Continue
                | Switch
                | Try
                | Throw
                | True
                | False
                | Null
                | Import
                | Export
                | While
        )
    }

    fn name_range_from_attr_name_node<'a>(node: &GreenNode<'a, VueLanguage>, offset: usize) -> Range<usize> {
        let mut current = offset;
        for child in node.children {
            match child {
                GreenTree::Leaf(t) if Self::is_name_token(t.kind) => {
                    return Range { start: current, end: current + t.length as usize };
                }
                GreenTree::Leaf(t) => current += t.length as usize,
                GreenTree::Node(n) => current += n.byte_length as usize,
            }
        }
        Range { start: offset, end: offset }
    }

    fn build_interpolation<'a>(&self, node: &GreenNode<'a, VueLanguage>, offset: usize, _source: &SourceText) -> Result<VueInterpolation, OakError> {
        let mut expression = Range { start: offset, end: offset };
        let mut current_offset = offset;

        for child in node.children {
            match child {
                GreenTree::Leaf(t) => {
                    if t.kind != VueTokenType::InterpolationStart && t.kind != VueTokenType::InterpolationEnd {
                        if expression.is_empty() {
                            expression = Range { start: current_offset, end: current_offset + t.length as usize };
                        }
                        else {
                            expression.end = current_offset + t.length as usize;
                        }
                    }
                    current_offset += t.length as usize;
                }
                GreenTree::Node(n) => {
                    if expression.is_empty() {
                        expression = Range { start: current_offset, end: current_offset + n.byte_length as usize };
                    }
                    else {
                        expression.end = current_offset + n.byte_length as usize;
                    }
                    current_offset += n.byte_length as usize;
                }
            }
        }

        let result = VueInterpolation { expression, span: Range { start: offset, end: offset + node.byte_length as usize } };
        Ok(result)
    }
}

impl Builder<VueLanguage> for VueBuilder {
    fn build<'a, S: Source + ?Sized>(&self, text: &S, edits: &[TextEdit], cache: &'a mut impl BuilderCache<VueLanguage>) -> BuildOutput<VueLanguage> {
        let source = SourceText::new(text.get_text_in(Range { start: 0, end: text.length() }).to_string());
        let config = VueLanguage::default();
        let parser = crate::parser::VueParser::new(&config);
        let parse_output = parser.parse(text, edits, cache);

        let mut diagnostics = Vec::new();
        for error in parse_output.diagnostics {
            diagnostics.push(error);
        }

        let result = parse_output.result.and_then(|green_tree| self.build_root(green_tree, &source));

        OakDiagnostics { result, diagnostics }
    }
}
