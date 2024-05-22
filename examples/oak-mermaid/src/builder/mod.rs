use crate::{
    ast::*,
    language::MermaidLanguage,
    lexer::token_type::MermaidTokenType,
    parser::{MermaidParser, element_type::MermaidElementType},
};
use oak_core::{
    Builder, BuilderCache, GreenNode, OakDiagnostics, Source, SourceText, TextEdit,
    tree::{GreenLeaf, GreenTree},
};
use std::collections::HashMap;

/// A builder for Mermaid Abstract Syntax Trees.
#[derive(Clone)]
pub struct MermaidBuilder<'config> {
    config: &'config MermaidLanguage,
}

impl<'config> MermaidBuilder<'config> {
    /// Creates a new Mermaid builder with the given configuration.
    pub fn new(config: &'config MermaidLanguage) -> Self {
        Self { config }
    }

    /// Builds a Mermaid root from a green tree and source text.
    pub fn build_root(&self, green_tree: &GreenNode<MermaidLanguage>, source: &SourceText) -> Result<MermaidRoot, oak_core::OakError> {
        let mut diagrams = Vec::new();
        let mut offset = 0usize;

        for child in green_tree.children {
            match child {
                GreenTree::Leaf(leaf) => offset += leaf.length as usize,
                GreenTree::Node(node) => {
                    if node.kind == MermaidElementType::Graph {
                        diagrams.push(MermaidDiagram::Flowchart(self.build_flowchart(node, source, &mut offset)));
                    }
                    else {
                        offset += node.byte_length as usize;
                    }
                }
            }
        }

        Ok(MermaidRoot { diagrams, span: (0..green_tree.byte_length as usize).into() })
    }

    /// 把一个 `Graph` 段落转换成 [`Flowchart`]。
    fn build_flowchart(&self, graph: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> Flowchart {
        let end = *offset + graph.byte_length as usize;
        let mut orientation = String::from("TD");
        let mut elements: Vec<FlowElement> = Vec::new();
        let mut node_index: HashMap<String, usize> = HashMap::new();

        for child in graph.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    let text = Self::leaf_text(*leaf, source, offset);
                    if leaf.kind == MermaidTokenType::Direction {
                        orientation = text;
                    }
                }
                GreenTree::Node(node) => match node.kind {
                    MermaidElementType::Node => {
                        let tokens = Self::collect_tokens(node, source, offset);
                        Self::append_node(&tokens, &mut elements, &mut node_index);
                    }
                    MermaidElementType::Edge => {
                        let tokens = Self::collect_tokens(node, source, offset);
                        Self::append_edges(&tokens, &mut elements, &mut node_index);
                    }
                    _ => *offset += node.byte_length as usize,
                },
            }
        }

        debug_assert_eq!(*offset, end);
        Flowchart { orientation, elements }
    }

    /// 读取一个 leaf 的文本，并把游标推进到下一个 token。
    fn leaf_text(leaf: GreenLeaf<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> String {
        let start = *offset;
        let end = start + leaf.length as usize;
        *offset = end;
        source.get_text_in((start..end).into()).into_owned()
    }

    /// 收集一个节点内部的全部 token（含 trivia），并按源码顺序返回。
    fn collect_tokens(node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> Vec<(MermaidTokenType, String)> {
        let mut tokens = Vec::new();
        Self::collect_tokens_into(node, source, offset, &mut tokens);
        tokens
    }

    fn collect_tokens_into(node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize, tokens: &mut Vec<(MermaidTokenType, String)>) {
        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => tokens.push((leaf.kind, Self::leaf_text(*leaf, source, offset))),
                GreenTree::Node(node) => Self::collect_tokens_into(node, source, offset, tokens),
            }
        }
    }

    /// 从 `Node` 元素的 token 序列中登记一个节点。
    fn append_node(tokens: &[(MermaidTokenType, String)], elements: &mut Vec<FlowElement>, node_index: &mut HashMap<String, usize>) {
        let significant: Vec<(MermaidTokenType, &str)> = tokens.iter().filter(|(kind, _)| !Self::is_trivia(*kind)).map(|(kind, text)| (*kind, text.as_str())).collect();

        let Some((MermaidTokenType::Id, id)) = significant.first().copied()
        else {
            return;
        };
        let label = significant.get(1).filter(|(kind, _)| *kind == MermaidTokenType::Label).map(|(_, text)| *text);
        Self::upsert_node(elements, node_index, id, label);
    }

    /// 从 `Edge` 元素的 token 序列中还原节点与边（支持链式 `A --> B --> C`）。
    fn append_edges(tokens: &[(MermaidTokenType, String)], elements: &mut Vec<FlowElement>, node_index: &mut HashMap<String, usize>) {
        let significant: Vec<(MermaidTokenType, &str)> = tokens.iter().filter(|(kind, _)| !Self::is_trivia(*kind)).map(|(kind, text)| (*kind, text.as_str())).collect();

        if significant.first().map(|(kind, _)| *kind) != Some(MermaidTokenType::Id) {
            return;
        }
        let mut index = 1usize;
        let mut from = significant[0].1;
        let from_label = significant.get(index).filter(|(kind, _)| *kind == MermaidTokenType::Label).map(|(_, text)| {
            index += 1;
            *text
        });
        Self::upsert_node(elements, node_index, from, from_label);

        while index < significant.len() {
            if significant[index].0 != MermaidTokenType::Arrow {
                break;
            }
            let mut arrow = significant[index].1;
            index += 1;

            let mut text = None;
            if arrow == "--" {
                // `-- text -->`：起始 `--` 与结束箭头之间的 token 就是边文本。
                let mut words = Vec::new();
                while index < significant.len() && significant[index].0 != MermaidTokenType::Arrow {
                    words.push(significant[index].1);
                    index += 1;
                }
                if !words.is_empty() {
                    text = Some(words.join(" "));
                }
                if index >= significant.len() {
                    break;
                }
                arrow = significant[index].1;
                index += 1;
            }
            else if let Some((MermaidTokenType::Label, label)) = significant.get(index).copied() {
                if label.starts_with('|') {
                    text = Some(Self::strip_edge_label(label));
                    index += 1;
                }
            }

            if index >= significant.len() || significant[index].0 != MermaidTokenType::Id {
                break;
            }
            let to = significant[index].1;
            index += 1;
            let to_label = significant.get(index).filter(|(kind, _)| *kind == MermaidTokenType::Label).map(|(_, text)| {
                index += 1;
                *text
            });
            Self::upsert_node(elements, node_index, to, to_label);

            elements.push(FlowElement::Edge { from: from.to_string(), to: to.to_string(), text, style: Self::edge_style(arrow) });
            from = to;
        }
    }

    /// 登记节点；同一 id 重复出现时保留第一次带标签的定义。
    fn upsert_node(elements: &mut Vec<FlowElement>, node_index: &mut HashMap<String, usize>, id: &str, label: Option<&str>) {
        let (shape, text) = match label {
            Some(label) => {
                let (shape, inner) = Self::derive_shape(label);
                (shape, Some(inner))
            }
            None => (NodeShape::Box, None),
        };

        if let Some(&position) = node_index.get(id) {
            if let FlowElement::Node { text: existing_text, shape: existing_shape, .. } = &mut elements[position] {
                if existing_text.is_none() && text.is_some() {
                    *existing_text = text;
                    *existing_shape = shape;
                }
            }
            return;
        }

        node_index.insert(id.to_string(), elements.len());
        elements.push(FlowElement::Node { id: id.to_string(), text, shape });
    }

    /// 由标签首尾定界符推导节点形状，并返回去掉定界符后的文本。
    fn derive_shape(label: &str) -> (NodeShape, String) {
        let label = label.trim();
        let shapes: &[(&str, &str, NodeShape)] =
            &[("[[", "]]", NodeShape::Subroutine), ("([", "])", NodeShape::Stadium), ("[(", ")]", NodeShape::Cylinder), ("((", "))", NodeShape::Circle), ("[", "]", NodeShape::Box), ("(", ")", NodeShape::Round), ("{", "}", NodeShape::Rhombus)];

        for (open, close, shape) in shapes {
            if let Some(inner) = label.strip_prefix(open).and_then(|rest| rest.strip_suffix(close)) {
                return (*shape, inner.trim().to_string());
            }
        }
        (NodeShape::Box, label.to_string())
    }

    /// 由箭头文本推导边样式。
    fn edge_style(arrow: &str) -> EdgeStyle {
        match arrow {
            "---" | "--" => EdgeStyle::Line,
            "-.->" => EdgeStyle::DottedArrow,
            "==>" => EdgeStyle::ThickArrow,
            _ => EdgeStyle::Arrow,
        }
    }

    /// 去掉边标签两侧的 `|`。
    fn strip_edge_label(label: &str) -> String {
        let label = label.trim();
        label.strip_prefix('|').and_then(|rest| rest.strip_suffix('|')).unwrap_or(label).trim().to_string()
    }

    /// trivia 不参与 AST 构建。
    fn is_trivia(kind: MermaidTokenType) -> bool {
        matches!(kind, MermaidTokenType::Whitespace | MermaidTokenType::Newline | MermaidTokenType::Comment | MermaidTokenType::Eof)
    }
}

impl<'config> Builder<MermaidLanguage> for MermaidBuilder<'config> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<MermaidLanguage>) -> oak_core::builder::BuildOutput<MermaidLanguage> {
        let parser = MermaidParser::new(self.config);
        let lexer = crate::lexer::MermaidLexer::new(self.config);
        let mut cache = oak_core::parser::session::ParseSession::<MermaidLanguage>::default();
        let parse_result = oak_core::parser::parse(&parser, &lexer, source, edits, &mut cache);

        match parse_result.result {
            Ok(green_tree) => {
                let source_text = SourceText::new(source.get_text_in((0..source.length()).into()).into_owned());
                OakDiagnostics { result: self.build_root(green_tree, &source_text), diagnostics: parse_result.diagnostics }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}
