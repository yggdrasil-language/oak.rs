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

/// sequence diagram 中可识别的消息箭头，较长的箭头排在前面。
const MESSAGE_ARROWS: &[(&str, MessageArrow)] = &[("-->>", MessageArrow::DottedArrow), ("->>", MessageArrow::SolidArrow), ("-->", MessageArrow::Dotted), ("->", MessageArrow::Solid)];

/// class diagram 中可识别的关系箭头，较长的箭头排在前面。
const RELATION_ARROWS: &[(&str, oak_uml::model::RelationKind)] = &[
    ("<|--", oak_uml::model::RelationKind::Inheritance),
    ("--|>", oak_uml::model::RelationKind::Inheritance),
    ("..|>", oak_uml::model::RelationKind::Inheritance),
    ("|>..", oak_uml::model::RelationKind::Inheritance),
    ("*--", oak_uml::model::RelationKind::Composition),
    ("--*", oak_uml::model::RelationKind::Composition),
    ("o--", oak_uml::model::RelationKind::Aggregation),
    ("--o", oak_uml::model::RelationKind::Aggregation),
    ("..>", oak_uml::model::RelationKind::Dependency),
    ("<..", oak_uml::model::RelationKind::Dependency),
    ("-->", oak_uml::model::RelationKind::Association),
    ("<--", oak_uml::model::RelationKind::Association),
    ("..", oak_uml::model::RelationKind::Dependency),
];

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
                    else if node.kind == MermaidElementType::SequenceDiagram {
                        diagrams.push(MermaidDiagram::Sequence(self.build_sequence(node, source, &mut offset)));
                    }
                    else if node.kind == MermaidElementType::ClassDiagram {
                        diagrams.push(MermaidDiagram::Class(self.build_class(node, source, &mut offset)));
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
                    let kind = leaf.kind;
                    let text = Self::leaf_text(*leaf, source, offset);
                    if kind == MermaidTokenType::Direction {
                        orientation = text;
                    }
                }
                GreenTree::Node(node) => self.append_element(node, source, offset, &mut elements, &mut node_index),
            }
        }

        debug_assert_eq!(*offset, end);
        Flowchart { orientation, elements }
    }

    /// 把一个 `Subgraph` 段落转换成 [`Subgraph`]。
    ///
    /// 头部行的 token（`subgraph` 关键字与标题）只收集到第一个换行，
    /// 块内元素按 flowchart 规则递归构建。
    fn build_subgraph(&self, node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> Subgraph {
        let end = *offset + node.byte_length as usize;
        let mut header: Vec<String> = Vec::new();
        let mut in_header = true;
        let mut elements: Vec<FlowElement> = Vec::new();
        let mut node_index: HashMap<String, usize> = HashMap::new();

        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    let kind = leaf.kind;
                    let text = Self::leaf_text(*leaf, source, offset);
                    if in_header {
                        if kind == MermaidTokenType::Newline {
                            in_header = false;
                        }
                        else if !Self::is_trivia(kind) && kind != MermaidTokenType::Eof {
                            header.push(text);
                        }
                    }
                }
                GreenTree::Node(child) => self.append_element(child, source, offset, &mut elements, &mut node_index),
            }
        }

        debug_assert_eq!(*offset, end);
        let (id, title) = Self::subgraph_header(&header);
        Subgraph { id, title, elements }
    }

    /// 把一个 `SequenceDiagram` 段落转换成 [`SequenceDiagram`]。
    fn build_sequence(&self, node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> SequenceDiagram {
        let end = *offset + node.byte_length as usize;
        let mut participants = Vec::new();
        let mut messages = Vec::new();

        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    *offset += leaf.length as usize;
                }
                GreenTree::Node(child) => match child.kind {
                    MermaidElementType::Participant => {
                        let text = Self::node_text(child, source, offset);
                        if let Some(participant) = Self::parse_participant(&text) {
                            participants.push(participant);
                        }
                    }
                    MermaidElementType::Message => {
                        let text = Self::node_text(child, source, offset);
                        if let Some(message) = Self::parse_message(&text) {
                            messages.push(message);
                        }
                    }
                    _ => *offset += child.byte_length as usize,
                },
            }
        }

        debug_assert_eq!(*offset, end);
        SequenceDiagram { participants, messages }
    }

    /// 从一行 participant 声明中还原 id、显示名与关键字。
    fn parse_participant(text: &str) -> Option<Participant> {
        let text = text.trim();
        let (kind, rest) = if let Some(rest) = Self::strip_keyword(text, "participant") {
            (ParticipantKind::Participant, rest)
        }
        else if let Some(rest) = Self::strip_keyword(text, "actor") {
            (ParticipantKind::Actor, rest)
        }
        else {
            return None;
        };

        let rest = rest.trim();
        let (id, display) = match rest.split_once(" as ") {
            Some((id, display)) => (id.trim(), Some(display.trim()).filter(|display| !display.is_empty()).map(str::to_string)),
            None => (rest, None),
        };
        if id.is_empty() {
            return None;
        }
        Some(Participant { id: id.to_string(), display, kind })
    }

    /// 从一行消息中还原发送方、接收方、箭头与文本。
    fn parse_message(text: &str) -> Option<Message> {
        let text = text.trim();
        let (head, body) = match text.split_once(':') {
            Some((head, body)) => (head, body.trim()),
            None => (text, ""),
        };

        let (index, arrow, kind) = Self::find_arrow(head, MESSAGE_ARROWS)?;
        let from = head[..index].trim();
        let to = head[index + arrow.len()..].trim();
        if from.is_empty() || to.is_empty() {
            return None;
        }
        Some(Message { from: from.to_string(), to: to.to_string(), text: body.to_string(), arrow: kind })
    }

    /// 把一个 `ClassDiagram` 段落转换成 [`ClassDiagram`]。
    fn build_class(&self, node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> ClassDiagram {
        let end = *offset + node.byte_length as usize;
        let mut classes = Vec::new();
        let mut relations = Vec::new();

        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    *offset += leaf.length as usize;
                }
                GreenTree::Node(child) => match child.kind {
                    MermaidElementType::ClassDeclaration => {
                        if let Some(class) = self.build_class_declaration(child, source, offset) {
                            classes.push(class);
                        }
                    }
                    MermaidElementType::Relation => {
                        let text = Self::node_text(child, source, offset);
                        if let Some(relation) = Self::parse_relation(&text) {
                            relations.push(relation);
                        }
                    }
                    _ => *offset += child.byte_length as usize,
                },
            }
        }

        debug_assert_eq!(*offset, end);
        ClassDiagram { classes, relations }
    }

    /// 把一个 `ClassDeclaration` 段落转换成 [`oak_uml::model::Class`]。
    fn build_class_declaration(&self, node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> Option<oak_uml::model::Class> {
        let end = *offset + node.byte_length as usize;
        let mut header = String::new();
        let mut in_header = true;
        let mut members = Vec::new();

        for child in node.children {
            match child {
                GreenTree::Leaf(leaf) => {
                    let text = Self::leaf_text(*leaf, source, offset);
                    if in_header {
                        if leaf.kind == MermaidTokenType::Newline {
                            in_header = false;
                        }
                        else {
                            header.push_str(&text);
                        }
                    }
                }
                GreenTree::Node(child) => match child.kind {
                    MermaidElementType::ClassMember => {
                        let text = Self::node_text(child, source, offset);
                        if let Some(member) = Self::parse_member(&text) {
                            members.push(member);
                        }
                    }
                    _ => *offset += child.byte_length as usize,
                },
            }
        }

        debug_assert_eq!(*offset, end);
        let name = Self::class_name(&header)?;
        Some(oak_uml::model::Class { name, members })
    }

    /// 从 `class Foo` / `class Foo {` 头部还原类名。
    fn class_name(header: &str) -> Option<String> {
        let rest = Self::strip_keyword(header.trim(), "class")?;
        let rest = rest.trim();
        let rest = rest.strip_suffix('{').unwrap_or(rest).trim();
        let name = rest.split(":::").next().unwrap_or(rest).trim();
        (!name.is_empty()).then(|| name.to_string())
    }

    /// 从一行成员声明中还原成员；带 `(` 的行视为方法。
    fn parse_member(text: &str) -> Option<oak_uml::model::Member> {
        let name = text.trim().trim_end_matches(';').trim();
        if name.is_empty() {
            return None;
        }
        if name.contains('(') { Some(oak_uml::model::Member::new_method(name)) } else { Some(oak_uml::model::Member::new_field(name)) }
    }

    /// 从一行关系中还原源、目标、关系类型与可选标签。
    fn parse_relation(text: &str) -> Option<oak_uml::model::Relation> {
        let text = text.trim();
        let (head, label) = match text.split_once(':') {
            Some((head, label)) => (head.trim(), Some(label.trim()).filter(|label| !label.is_empty()).map(str::to_string)),
            None => (text, None),
        };

        let (index, arrow, kind) = Self::find_arrow(head, RELATION_ARROWS)?;
        let source = head[..index].trim();
        let target = head[index + arrow.len()..].trim();
        if source.is_empty() || target.is_empty() {
            return None;
        }
        Some(oak_uml::model::Relation { source: source.to_string(), target: target.to_string(), kind, label })
    }

    /// 在文本中定位最靠左的箭头；同一位置优先匹配列表靠前的较长箭头。
    fn find_arrow<'a, T: Clone>(text: &str, arrows: &'a [(&'a str, T)]) -> Option<(usize, &'a str, T)> {
        for (index, _) in text.char_indices() {
            for (arrow, kind) in arrows {
                if text[index..].starts_with(arrow) {
                    return Some((index, arrow, kind.clone()));
                }
            }
        }
        None
    }

    /// 去掉关键字前缀，并确保后面跟着空白或字符串结束。
    fn strip_keyword<'a>(text: &'a str, keyword: &str) -> Option<&'a str> {
        let rest = text.strip_prefix(keyword)?;
        if rest.is_empty() || rest.starts_with(char::is_whitespace) { Some(rest) } else { None }
    }

    /// 把一个子节点追加到元素列表。
    fn append_element(&self, node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize, elements: &mut Vec<FlowElement>, node_index: &mut HashMap<String, usize>) {
        match node.kind {
            MermaidElementType::Node => {
                let tokens = Self::collect_tokens(node, source, offset);
                Self::append_node(&tokens, elements, node_index);
            }
            MermaidElementType::Edge => {
                let tokens = Self::collect_tokens(node, source, offset);
                Self::append_edges(&tokens, elements, node_index);
            }
            MermaidElementType::Subgraph => {
                let subgraph = self.build_subgraph(node, source, offset);
                elements.push(FlowElement::Subgraph(subgraph));
            }
            _ => *offset += node.byte_length as usize,
        }
    }

    /// 从头部 token 解析子图 id 与标题。
    ///
    /// 支持 `subgraph title` 与 `subgraph id[title]` 两种写法。
    fn subgraph_header(header: &[String]) -> (Option<String>, String) {
        let mut tokens = header.iter().skip(1);
        let Some(first) = tokens.next()
        else {
            return (None, String::new());
        };
        let rest: Vec<&str> = tokens.map(String::as_str).collect();
        if rest.is_empty() {
            let title = Self::strip_brackets(first);
            return (None, title);
        }
        (Some(first.clone()), Self::strip_brackets(&rest.join(" ")))
    }

    /// 去掉标题两侧的 `[..]` 或引号。
    fn strip_brackets(text: &str) -> String {
        let text = text.trim();
        for (open, close) in [("[", "]"), ("\"", "\"")] {
            if let Some(inner) = text.strip_prefix(open).and_then(|rest| rest.strip_suffix(close)) {
                return inner.trim().to_string();
            }
        }
        text.to_string()
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

    /// 还原一个节点的原始行文本（保留 token 之间的空白）。
    fn node_text(node: &GreenNode<MermaidLanguage>, source: &SourceText, offset: &mut usize) -> String {
        Self::collect_tokens(node, source, offset).into_iter().map(|(_, text)| text).collect()
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
