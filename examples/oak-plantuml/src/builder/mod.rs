//! Builder that lowers the PlantUML green tree into the typed AST.

use crate::{
    ast::{ActivationSpan, ClassDeclaration, ClassDiagram, ClassKeyword, ClassMember, ClassRelation, ClassRelationKind, MemberVisibility, MessageArrow, Participant, ParticipantKind, PlantUmlDiagram, PlantUmlRoot, SequenceDiagram, SequenceMessage},
    language::PlantUmlLanguage,
    lexer::token_type::PlantUmlTokenType,
    parser::element_type::PlantUmlElementType,
};
use oak_core::{Builder, BuilderCache, GreenNode, OakDiagnostics, OakError, Parser, RedLeaf, RedNode, RedTree, SourceText, TextEdit, source::Source};
use std::collections::HashMap;

/// Builder for converting a PlantUML syntax tree into the typed AST.
pub struct PlantUmlBuilder<'config> {
    /// The PlantUML language configuration.
    _config: &'config PlantUmlLanguage,
}

impl<'config> PlantUmlBuilder<'config> {
    /// Creates a builder with the given language configuration.
    pub fn new(config: &'config PlantUmlLanguage) -> Self {
        Self { _config: config }
    }
}

impl Builder<PlantUmlLanguage> for PlantUmlBuilder<'_> {
    fn build<'a, S: Source + ?Sized>(&self, source: &S, edits: &[TextEdit], _cache: &'a mut impl BuilderCache<PlantUmlLanguage>) -> OakDiagnostics<PlantUmlRoot> {
        let parser = crate::parser::PlantUmlParser::new(self._config);
        let mut cache = oak_core::parser::session::ParseSession::<PlantUmlLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut cache);

        match parse_result.result {
            Ok(green_tree) => {
                let source_text = SourceText::new(source.get_text_in((0..source.length()).into()).into_owned());
                let mut diagnostics = parse_result.diagnostics;
                match self.build_root(green_tree, &source_text, &mut diagnostics) {
                    Ok(ast_root) => OakDiagnostics { result: Ok(ast_root), diagnostics },
                    Err(build_error) => {
                        diagnostics.push(build_error.clone());
                        OakDiagnostics { result: Err(build_error), diagnostics }
                    }
                }
            }
            Err(parse_error) => OakDiagnostics { result: Err(parse_error), diagnostics: parse_result.diagnostics },
        }
    }
}

impl PlantUmlBuilder<'_> {
    /// Builds the AST root from a green tree.
    pub(crate) fn build_root(&self, green_tree: &GreenNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Result<PlantUmlRoot, OakError> {
        if green_tree.kind != PlantUmlElementType::Root {
            return Err(OakError::internal_error("the PlantUML syntax tree is missing its root element"));
        }

        let red_root = RedNode::new(green_tree, 0);
        let mut elements = Vec::new();
        let mut start = None;
        let mut end = None;

        for child in red_root.children() {
            let RedTree::Node(node) = child
            else {
                continue;
            };

            match node.green.kind {
                PlantUmlElementType::StartMarker => {
                    start.get_or_insert(node.span().start);
                    end = Some(node.span().end);
                }
                PlantUmlElementType::EndMarker => {
                    end = Some(node.span().end);
                }
                PlantUmlElementType::ClassDeclaration | PlantUmlElementType::ClassRelation | PlantUmlElementType::ParticipantDeclaration | PlantUmlElementType::SequenceMessage | PlantUmlElementType::Activate | PlantUmlElementType::Deactivate => {
                    start.get_or_insert(node.span().start);
                    end = Some(node.span().end);
                    elements.push(node);
                }
                PlantUmlElementType::Comment | PlantUmlElementType::Error => {
                    if start.is_some() {
                        end = Some(node.span().end);
                    }
                }
                _ => {
                    diagnostics.push(source.syntax_error(format!("unexpected element in PlantUML root: `{:?}`", node.green.kind), node.span().start));
                }
            }
        }

        let Some(span_start) = start
        else {
            return Ok(PlantUmlRoot { diagrams: Vec::new() });
        };
        let span = core::range::Range { start: span_start, end: end.unwrap_or(span_start) };

        let is_sequence = elements.iter().any(|node| matches!(node.green.kind, PlantUmlElementType::ParticipantDeclaration | PlantUmlElementType::SequenceMessage | PlantUmlElementType::Activate | PlantUmlElementType::Deactivate));

        let diagram = if is_sequence { PlantUmlDiagram::Sequence(self.build_sequence_diagram(&elements, source, diagnostics, span)) } else { PlantUmlDiagram::Class(self.build_class_diagram(&elements, source, diagnostics, span)) };

        Ok(PlantUmlRoot { diagrams: vec![diagram] })
    }

    fn build_class_diagram(&self, elements: &[RedNode<PlantUmlLanguage>], source: &SourceText, diagnostics: &mut Vec<OakError>, span: core::range::Range<usize>) -> ClassDiagram {
        let mut classes = Vec::new();
        let mut relations = Vec::new();

        for node in elements {
            match node.green.kind {
                PlantUmlElementType::ClassDeclaration => classes.push(self.build_class_declaration(*node, source, diagnostics)),
                PlantUmlElementType::ClassRelation => {
                    if let Some(relation) = self.build_class_relation(*node, source, diagnostics) {
                        relations.push(relation);
                    }
                }
                _ => {}
            }
        }

        ClassDiagram { classes, relations, span }
    }

    fn build_class_declaration(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> ClassDeclaration {
        let span = node.span();
        let leaves = significant_leaves(&node);
        let mut index = 0;
        let mut keyword = ClassKeyword::Class;

        match leaves.first().map(|leaf| leaf.kind) {
            Some(PlantUmlTokenType::Abstract) => {
                keyword = ClassKeyword::Abstract;
                index = 1;
                if leaves.get(1).map(|leaf| leaf.kind) == Some(PlantUmlTokenType::Class) {
                    index = 2;
                }
            }
            Some(PlantUmlTokenType::Interface) => {
                keyword = ClassKeyword::Interface;
                index = 1;
            }
            Some(PlantUmlTokenType::Enum) => {
                keyword = ClassKeyword::Enum;
                index = 1;
            }
            Some(PlantUmlTokenType::Class) => {
                index = 1;
            }
            _ => {}
        }

        let mut name = String::new();
        let mut display = None;

        if let Some(leaf) = leaves.get(index) {
            if leaf.kind == PlantUmlTokenType::String {
                let value = unquote(leaf.text(source).as_ref());
                display = Some(value.clone());
                name = value;
                index += 1;
            }
            else if is_name_kind(leaf.kind) {
                name = leaf.text(source).into_owned();
                index += 1;
            }
        }

        if leaves.get(index).map(|leaf| leaf.kind) == Some(PlantUmlTokenType::As) {
            if let Some(alias) = leaves.get(index + 1) {
                if is_name_kind(alias.kind) {
                    name = alias.text(source).into_owned();
                }
                else {
                    diagnostics.push(source.syntax_error("expected an alias after `as`", alias.span.start));
                }
            }
        }

        let mut members = Vec::new();
        for child in node.children() {
            if let RedTree::Node(child) = child {
                if child.green.kind == PlantUmlElementType::ClassMemberBlock {
                    members = self.build_class_members(child, source, diagnostics);
                }
            }
        }

        ClassDeclaration { name, display, keyword, members, span }
    }

    fn build_class_members(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Vec<ClassMember> {
        let mut members = Vec::new();

        for child in node.children() {
            if let RedTree::Node(child) = child {
                if child.green.kind == PlantUmlElementType::ClassMember {
                    members.push(self.build_class_member(child, source, diagnostics));
                }
            }
        }

        members
    }

    fn build_class_member(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, _diagnostics: &mut Vec<OakError>) -> ClassMember {
        let span = node.span();
        let leaves = significant_leaves(&node);
        let visibility = match leaves.first().map(|leaf| leaf.kind) {
            Some(PlantUmlTokenType::Minus) => MemberVisibility::Private,
            Some(PlantUmlTokenType::Hash) => MemberVisibility::Protected,
            _ => MemberVisibility::Public,
        };
        let text_start = leaves.first().map(|leaf| leaf.span.end).unwrap_or(span.start);
        let text = source.get_text_in(core::range::Range { start: text_start, end: span.end }).trim().to_string();
        let is_method = text.contains('(');

        ClassMember { visibility, text, is_method, span }
    }

    fn build_class_relation(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Option<ClassRelation> {
        let span = node.span();
        let leaves = significant_leaves(&node);
        let Some(left_leaf) = leaves.first()
        else {
            diagnostics.push(source.syntax_error("a class relation is missing its left endpoint", span.start));
            return None;
        };
        let left = unquote(left_leaf.text(source).as_ref());
        let mut index = 1;

        let left_multiplicity = if leaves.get(index).map(|leaf| leaf.kind) == Some(PlantUmlTokenType::String) {
            let value = unquote(leaves[index].text(source).as_ref());
            index += 1;
            Some(value)
        }
        else {
            None
        };

        let Some(arrow_leaf) = leaves.get(index)
        else {
            diagnostics.push(source.syntax_error("a class relation is missing its arrow", span.start));
            return None;
        };
        if arrow_leaf.kind != PlantUmlTokenType::Arrow {
            diagnostics.push(source.syntax_error("a class relation is missing its arrow", arrow_leaf.span.start));
            return None;
        }
        let arrow = arrow_leaf.text(source).into_owned();
        let Some((kind, marker_at_left)) = relation_arrow(&arrow)
        else {
            diagnostics.push(source.syntax_error(format!("unsupported class relation arrow `{arrow}`"), arrow_leaf.span.start));
            return None;
        };
        index += 1;

        let right_multiplicity = if leaves.get(index).map(|leaf| leaf.kind) == Some(PlantUmlTokenType::String) {
            let value = unquote(leaves[index].text(source).as_ref());
            index += 1;
            Some(value)
        }
        else {
            None
        };

        let Some(right_leaf) = leaves.get(index)
        else {
            diagnostics.push(source.syntax_error("a class relation is missing its right endpoint", span.start));
            return None;
        };
        if !is_name_kind(right_leaf.kind) {
            diagnostics.push(source.syntax_error("a class relation is missing its right endpoint", right_leaf.span.start));
            return None;
        }
        let right = unquote(right_leaf.text(source).as_ref());
        index += 1;

        let label = if leaves.get(index).map(|leaf| leaf.kind) == Some(PlantUmlTokenType::Colon) {
            let colon = leaves[index];
            let text = source.get_text_in(core::range::Range { start: colon.span.end, end: span.end }).trim().to_string();
            if text.is_empty() { None } else { Some(text) }
        }
        else {
            None
        };

        Some(ClassRelation { left, right, left_multiplicity, right_multiplicity, kind, marker_at_left, label, span })
    }

    fn build_sequence_diagram(&self, elements: &[RedNode<PlantUmlLanguage>], source: &SourceText, diagnostics: &mut Vec<OakError>, span: core::range::Range<usize>) -> SequenceDiagram {
        let participants: Vec<Participant> = elements.iter().filter(|node| node.green.kind == PlantUmlElementType::ParticipantDeclaration).map(|node| self.build_participant(*node, source, diagnostics)).collect();

        let mut names = HashMap::new();
        for participant in &participants {
            names.insert(participant.id.clone(), participant.id.clone());
            if let Some(display) = &participant.display {
                names.entry(display.clone()).or_insert_with(|| participant.id.clone());
            }
        }

        let mut messages = Vec::new();
        let mut activations = Vec::new();
        let mut open: Vec<(String, usize)> = Vec::new();

        for node in elements {
            match node.green.kind {
                PlantUmlElementType::SequenceMessage => messages.push(self.build_sequence_message(*node, source, &names, diagnostics)),
                PlantUmlElementType::Activate => {
                    if let Some(name) = activation_name(*node, source) {
                        if !open.iter().any(|(active, _)| active == &name) {
                            open.push((name, messages.len()));
                        }
                    }
                }
                PlantUmlElementType::Deactivate => {
                    if let Some(name) = activation_name(*node, source) {
                        if let Some(position) = open.iter().rposition(|(active, _)| active == &name) {
                            let (participant, from_message) = open.remove(position);
                            activations.push(ActivationSpan { participant, from_message, to_message: messages.len().saturating_sub(1) });
                        }
                    }
                }
                _ => {}
            }
        }

        if !messages.is_empty() {
            for (participant, from_message) in open {
                activations.push(ActivationSpan { participant, from_message, to_message: messages.len() - 1 });
            }
        }
        activations.sort_by_key(|activation| activation.from_message);

        SequenceDiagram { participants, messages, activations, span }
    }

    fn build_participant(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>) -> Participant {
        let span = node.span();
        let leaves = significant_leaves(&node);
        let kind = match leaves.first().map(|leaf| leaf.kind) {
            Some(PlantUmlTokenType::Actor) => ParticipantKind::Actor,
            _ => ParticipantKind::Participant,
        };
        let mut index = 1;
        let mut display = None;

        let mut id = if let Some(leaf) = leaves.get(index) {
            if leaf.kind == PlantUmlTokenType::String {
                let value = unquote(leaf.text(source).as_ref());
                display = Some(value.clone());
                index += 1;
                value
            }
            else if is_name_kind(leaf.kind) {
                index += 1;
                leaf.text(source).into_owned()
            }
            else {
                diagnostics.push(source.syntax_error("expected a participant name", leaf.span.start));
                String::new()
            }
        }
        else {
            diagnostics.push(source.syntax_error("expected a participant name", span.start));
            String::new()
        };

        if leaves.get(index).map(|leaf| leaf.kind) == Some(PlantUmlTokenType::As) {
            if let Some(alias) = leaves.get(index + 1) {
                if is_name_kind(alias.kind) {
                    id = alias.text(source).into_owned();
                }
                else {
                    diagnostics.push(source.syntax_error("expected an alias after `as`", alias.span.start));
                }
            }
        }

        Participant { id, display, kind, span }
    }

    fn build_sequence_message(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, names: &HashMap<String, String>, diagnostics: &mut Vec<OakError>) -> SequenceMessage {
        let span = node.span();
        let leaves = significant_leaves(&node);
        let from_leaf = leaves.first();
        let arrow_leaf = leaves.get(1);
        let to_leaf = leaves.get(2);

        let from = from_leaf.map(|leaf| resolve_endpoint(&unquote(leaf.text(source).as_ref()), names)).unwrap_or_default();
        let to = to_leaf.map(|leaf| resolve_endpoint(&unquote(leaf.text(source).as_ref()), names)).unwrap_or_default();

        let arrow = match arrow_leaf.map(|leaf| leaf.text(source).into_owned()).as_deref() {
            Some("->") => MessageArrow::Solid,
            Some("->>") => MessageArrow::SolidArrow,
            Some("-->") => MessageArrow::Dotted,
            Some("-->>") => MessageArrow::DottedArrow,
            Some(other) => {
                diagnostics.push(source.syntax_error(format!("unsupported sequence message arrow `{other}`"), span.start));
                MessageArrow::Solid
            }
            None => {
                diagnostics.push(source.syntax_error("a sequence message is missing its arrow", span.start));
                MessageArrow::Solid
            }
        };

        let text = leaves.iter().find(|leaf| leaf.kind == PlantUmlTokenType::Colon).map(|colon| source.get_text_in(core::range::Range { start: colon.span.end, end: span.end }).trim().to_string()).filter(|text| !text.is_empty());

        SequenceMessage { from, to, text, arrow, span }
    }
}

fn significant_leaves(node: &RedNode<PlantUmlLanguage>) -> Vec<RedLeaf<PlantUmlLanguage>> {
    node.children()
        .filter_map(|child| match child {
            RedTree::Leaf(leaf) if leaf.kind != PlantUmlTokenType::Whitespace => Some(leaf),
            _ => None,
        })
        .collect()
}

fn is_name_kind(kind: PlantUmlTokenType) -> bool {
    matches!(kind, PlantUmlTokenType::Identifier | PlantUmlTokenType::String | PlantUmlTokenType::Text)
}

fn unquote(text: &str) -> String {
    let trimmed = text.trim();
    let Some(inner) = trimmed.strip_prefix('"').and_then(|value| value.strip_suffix('"'))
    else {
        return trimmed.to_string();
    };

    let mut result = String::with_capacity(inner.len());
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            result.push(ch);
            escaped = false;
        }
        else if ch == '\\' {
            escaped = true;
        }
        else {
            result.push(ch);
        }
    }
    if escaped {
        result.push('\\');
    }
    result
}

fn relation_arrow(arrow: &str) -> Option<(ClassRelationKind, bool)> {
    Some(match arrow {
        "-->" => (ClassRelationKind::Association, false),
        "<--" => (ClassRelationKind::Association, true),
        "<|--" => (ClassRelationKind::Inheritance, true),
        "--|>" => (ClassRelationKind::Inheritance, false),
        "*--" => (ClassRelationKind::Composition, true),
        "--*" => (ClassRelationKind::Composition, false),
        "o--" => (ClassRelationKind::Aggregation, true),
        "--o" => (ClassRelationKind::Aggregation, false),
        "..>" => (ClassRelationKind::Dependency, false),
        "<.." => (ClassRelationKind::Dependency, true),
        ".." => (ClassRelationKind::Link, false),
        _ => return None,
    })
}

fn activation_name(node: RedNode<PlantUmlLanguage>, source: &SourceText) -> Option<String> {
    significant_leaves(&node).get(1).map(|leaf| leaf.text(source).into_owned())
}

fn resolve_endpoint(raw: &str, names: &HashMap<String, String>) -> String {
    names.get(raw).cloned().unwrap_or_else(|| raw.to_string())
}
