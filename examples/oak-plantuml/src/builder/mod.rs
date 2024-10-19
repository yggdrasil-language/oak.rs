//! Builder that lowers the PlantUML green tree into the typed AST.

use crate::{
    ast::{
        ActivationSpan, ActivityAction, ActivityBranch, ActivityBranchKind, ActivityDecision, ActivityDiagram, ActivityEndpoint, ActivityFork, ActivityForkBranch, ActivityNode, ActivitySwimlane, ActivityTransition, ClassDeclaration, ClassDiagram,
        ClassKeyword, ClassMember, ClassRelation, ClassRelationKind, ComponentDiagram, ComponentElement, ComponentNode, ComponentPackage, ComponentPort, ComponentRelation, MemberVisibility, MessageArrow, Participant, ParticipantKind, PlantUmlDiagram,
        PlantUmlRoot, PortDirection, SequenceDiagram, SequenceMessage, StateDefinition, StateDiagram, StateElement, StateEndpoint, StateKind, StateRegion, StateTransition,
    },
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
        let mut container = None;
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
                PlantUmlElementType::ActivityDiagram | PlantUmlElementType::StateDiagram | PlantUmlElementType::ComponentDiagram => {
                    start.get_or_insert(node.span().start);
                    end = Some(node.span().end);
                    container = Some(node);
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

        let diagram = match container {
            Some(node) => match node.green.kind {
                PlantUmlElementType::ActivityDiagram => PlantUmlDiagram::Activity(self.build_activity_diagram(node, source, diagnostics, span)),
                PlantUmlElementType::StateDiagram => PlantUmlDiagram::State(self.build_state_diagram(node, source, diagnostics, span)),
                PlantUmlElementType::ComponentDiagram => PlantUmlDiagram::Component(self.build_component_diagram(node, source, diagnostics, span)),
                _ => unreachable!("only diagram containers reach this branch"),
            },
            None => {
                let is_sequence = elements.iter().any(|node| matches!(node.green.kind, PlantUmlElementType::ParticipantDeclaration | PlantUmlElementType::SequenceMessage | PlantUmlElementType::Activate | PlantUmlElementType::Deactivate));
                if is_sequence { PlantUmlDiagram::Sequence(self.build_sequence_diagram(&elements, source, diagnostics, span)) } else { PlantUmlDiagram::Class(self.build_class_diagram(&elements, source, diagnostics, span)) }
            }
        };

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
            Some(PlantUmlTokenType::Tilde) => MemberVisibility::Package,
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

    fn build_activity_diagram(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>, span: core::range::Range<usize>) -> ActivityDiagram {
        let lines = collect_lines(&node, source);
        let mut index = 0;
        let nodes = self.build_activity_nodes(&lines, &mut index, &[], false, diagnostics);
        ActivityDiagram { nodes, span }
    }

    fn build_activity_nodes(&self, lines: &[StatementLine], index: &mut usize, terminators: &[&str], stop_at_lane: bool, diagnostics: &mut Vec<OakError>) -> Vec<ActivityNode> {
        let mut nodes = Vec::new();

        while *index < lines.len() {
            let line = &lines[*index];
            let text = line.text.as_str();
            let head = first_word(text);

            if stop_at_lane && text.starts_with('|') {
                break;
            }
            if matches_terminator(text, terminators) {
                break;
            }

            if text.starts_with(':') {
                nodes.push(ActivityNode::Action(ActivityAction { text: action_text(text), span: line.span }));
                *index += 1;
            }
            else if text.contains("-->") {
                if let Some(transition) = parse_activity_transition(line) {
                    nodes.push(ActivityNode::Transition(transition));
                }
                *index += 1;
            }
            else {
                match head.as_str() {
                    "start" => {
                        nodes.push(ActivityNode::Start);
                        *index += 1;
                    }
                    "stop" => {
                        nodes.push(ActivityNode::Stop);
                        *index += 1;
                    }
                    "end" if text == "end" => {
                        nodes.push(ActivityNode::Stop);
                        *index += 1;
                    }
                    "if" => nodes.push(ActivityNode::Decision(self.build_activity_decision(lines, index, diagnostics))),
                    "fork" => nodes.push(ActivityNode::Fork(self.build_activity_fork(lines, index, diagnostics))),
                    _ if text.starts_with('|') => nodes.push(ActivityNode::Swimlane(self.build_activity_swimlane(lines, index, terminators, diagnostics))),
                    _ => *index += 1,
                }
            }
        }

        nodes
    }

    fn build_activity_decision(&self, lines: &[StatementLine], index: &mut usize, diagnostics: &mut Vec<OakError>) -> ActivityDecision {
        let start = lines[*index].span.start;
        let condition = parenthesized(&lines[*index].text);
        let label = parenthesized_after(&lines[*index].text, "then");
        *index += 1;

        let mut branches = Vec::new();
        let nodes = self.build_activity_nodes(lines, index, &["elseif", "else", "endif"], false, diagnostics);
        let end = consumed_end(lines, *index, start);
        branches.push(ActivityBranch { kind: ActivityBranchKind::Then, condition, label, nodes, span: core::range::Range { start, end } });

        loop {
            let Some(line) = lines.get(*index)
            else {
                diagnostics.push(OakError::syntax_error("unterminated activity decision, expected `endif`".to_string(), start, None));
                break;
            };
            let normalized = line.text.to_ascii_lowercase();
            let branch_start = line.span.start;

            if normalized.starts_with("elseif") {
                let condition = parenthesized(&line.text);
                let label = parenthesized_after(&line.text, "then");
                *index += 1;
                let nodes = self.build_activity_nodes(lines, index, &["elseif", "else", "endif"], false, diagnostics);
                let end = consumed_end(lines, *index, branch_start);
                branches.push(ActivityBranch { kind: ActivityBranchKind::ElseIf, condition, label, nodes, span: core::range::Range { start: branch_start, end } });
            }
            else if normalized.starts_with("else") {
                let label = parenthesized(&line.text);
                *index += 1;
                let nodes = self.build_activity_nodes(lines, index, &["elseif", "else", "endif"], false, diagnostics);
                let end = consumed_end(lines, *index, branch_start);
                branches.push(ActivityBranch { kind: ActivityBranchKind::Else, condition: None, label, nodes, span: core::range::Range { start: branch_start, end } });
            }
            else if normalized.starts_with("endif") {
                *index += 1;
                break;
            }
            else {
                break;
            }
        }

        let end = consumed_end(lines, *index, start);
        ActivityDecision { branches, span: core::range::Range { start, end } }
    }

    fn build_activity_fork(&self, lines: &[StatementLine], index: &mut usize, diagnostics: &mut Vec<OakError>) -> ActivityFork {
        let start = lines[*index].span.start;
        *index += 1;
        let mut branches = Vec::new();

        loop {
            let branch_start = lines.get(*index).map(|line| line.span.start).unwrap_or(start);
            let nodes = self.build_activity_nodes(lines, index, &["fork again", "end fork", "end merge", "end split"], false, diagnostics);
            let end = consumed_end(lines, *index, branch_start);
            branches.push(ActivityForkBranch { nodes, span: core::range::Range { start: branch_start, end } });

            let Some(line) = lines.get(*index)
            else {
                diagnostics.push(OakError::syntax_error("unterminated activity fork, expected `end fork`".to_string(), start, None));
                break;
            };
            let normalized = line.text.to_ascii_lowercase();
            if normalized == "fork again" {
                *index += 1;
                continue;
            }
            if matches!(normalized.as_str(), "end fork" | "end merge" | "end split") {
                *index += 1;
                break;
            }
            break;
        }

        let end = consumed_end(lines, *index, start);
        ActivityFork { branches, span: core::range::Range { start, end } }
    }

    fn build_activity_swimlane(&self, lines: &[StatementLine], index: &mut usize, terminators: &[&str], diagnostics: &mut Vec<OakError>) -> ActivitySwimlane {
        let line = &lines[*index];
        let span = line.span;
        let (name, color) = parse_swimlane(&line.text);
        *index += 1;
        let nodes = self.build_activity_nodes(lines, index, terminators, true, diagnostics);
        let end = consumed_end(lines, *index, span.end);
        ActivitySwimlane { name, color, nodes, span: core::range::Range { start: span.start, end } }
    }

    fn build_state_diagram(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>, span: core::range::Range<usize>) -> StateDiagram {
        let lines = collect_lines(&node, source);
        let mut index = 0;
        let mut elements = Vec::new();

        while index < lines.len() {
            let text = lines[index].text.as_str();
            if text == "}" || text == "--" {
                index += 1;
                continue;
            }
            if let Some(element) = self.parse_state_element(&lines, &mut index, diagnostics) {
                elements.push(element);
            }
        }

        StateDiagram { elements, span }
    }

    fn parse_state_element(&self, lines: &[StatementLine], index: &mut usize, diagnostics: &mut Vec<OakError>) -> Option<StateElement> {
        let line = &lines[*index];
        let text = line.text.as_str();

        if first_word(text) == "state" {
            *index += 1;
            let (id, display, kind, has_block) = parse_state_header(text, diagnostics, line.span.start);
            let regions = if has_block { self.build_state_regions(lines, index, diagnostics) } else { Vec::new() };
            return Some(StateElement::State(StateDefinition { id, text: display, kind, regions }));
        }

        if let Some(transition) = parse_state_transition(text) {
            *index += 1;
            return Some(StateElement::Transition(transition));
        }

        *index += 1;
        None
    }

    fn build_state_regions(&self, lines: &[StatementLine], index: &mut usize, diagnostics: &mut Vec<OakError>) -> Vec<StateRegion> {
        let mut regions = Vec::new();
        let mut elements = Vec::new();

        while *index < lines.len() {
            let text = lines[*index].text.as_str();
            if text == "}" {
                *index += 1;
                break;
            }
            if text == "--" {
                regions.push(StateRegion { elements: core::mem::take(&mut elements) });
                *index += 1;
                continue;
            }
            if let Some(element) = self.parse_state_element(lines, index, diagnostics) {
                elements.push(element);
            }
        }

        regions.push(StateRegion { elements });
        regions
    }

    fn build_component_diagram(&self, node: RedNode<PlantUmlLanguage>, source: &SourceText, diagnostics: &mut Vec<OakError>, span: core::range::Range<usize>) -> ComponentDiagram {
        let lines = collect_lines(&node, source);
        let mut index = 0;
        let mut elements = Vec::new();
        let mut relations = Vec::new();

        self.parse_component_block(&lines, &mut index, false, &mut elements, &mut relations, diagnostics);

        ComponentDiagram { elements, relations, span }
    }

    fn parse_component_block(&self, lines: &[StatementLine], index: &mut usize, in_block: bool, elements: &mut Vec<ComponentElement>, relations: &mut Vec<ComponentRelation>, diagnostics: &mut Vec<OakError>) {
        while *index < lines.len() {
            let line = &lines[*index];
            let text = line.text.as_str();

            if text == "}" {
                *index += 1;
                if in_block {
                    return;
                }
                continue;
            }

            let head = first_word(text);
            match head.as_str() {
                "package" => {
                    let (name, display, alias, has_block) = parse_named_header(head_word_len(text), text);
                    *index += 1;
                    let mut nested = Vec::new();
                    if has_block {
                        self.parse_component_block(lines, index, true, &mut nested, relations, diagnostics);
                    }
                    let end = consumed_end(lines, *index, line.span.end);
                    elements.push(ComponentElement::Package(ComponentPackage { name, display, alias, elements: nested, span: core::range::Range { start: line.span.start, end } }));
                }
                "component" | "interface" => {
                    let keyword = head.clone();
                    let (name, display, alias, has_block) = parse_named_header(head_word_len(text), text);
                    *index += 1;
                    let node = ComponentNode { name, display, alias, span: line.span };
                    if keyword == "interface" {
                        elements.push(ComponentElement::Interface(node));
                    }
                    else {
                        elements.push(ComponentElement::Component(node));
                    }
                    if has_block {
                        self.parse_component_block(lines, index, true, elements, relations, diagnostics);
                    }
                }
                "port" | "portin" | "portout" => {
                    let name = first_word(text[head.len()..].trim()).to_string();
                    let direction = match head.as_str() {
                        "portin" => PortDirection::In,
                        "portout" => PortDirection::Out,
                        _ => PortDirection::InOut,
                    };
                    elements.push(ComponentElement::Port(ComponentPort { name, direction, span: line.span }));
                    *index += 1;
                }
                _ if text.starts_with('[') => {
                    let (name, display, alias, _) = parse_bracket_header(text);
                    elements.push(ComponentElement::Component(ComponentNode { name, display, alias, span: line.span }));
                    *index += 1;
                }
                _ => {
                    if let Some(relation) = self.parse_component_relation(line, diagnostics) {
                        relations.push(relation);
                    }
                    *index += 1;
                }
            }
        }
    }

    fn parse_component_relation(&self, line: &StatementLine, diagnostics: &mut Vec<OakError>) -> Option<ComponentRelation> {
        let mut rest = line.text.as_str();
        let left = take_endpoint(&mut rest)?;
        let left_multiplicity = take_multiplicity(&mut rest);

        let trimmed = rest.trim_start();
        let arrow = find_arrow(trimmed)?;
        rest = &trimmed[arrow.len()..];
        let right_multiplicity = take_multiplicity(&mut rest);
        let right = take_endpoint(&mut rest)?;

        let Some((kind, marker_at_left)) = relation_arrow(arrow)
        else {
            diagnostics.push(OakError::syntax_error(format!("unsupported component relation arrow `{arrow}`"), line.span.start, None));
            return None;
        };

        let (_, label) = split_label(rest);
        Some(ComponentRelation { left, right, left_multiplicity, right_multiplicity, kind, marker_at_left, label, span: line.span })
    }
}

/// One significant statement line of a line-oriented PlantUML diagram.
struct StatementLine {
    span: core::range::Range<usize>,
    text: String,
}

fn collect_lines(node: &RedNode<PlantUmlLanguage>, source: &SourceText) -> Vec<StatementLine> {
    node.children()
        .filter_map(|child| match child {
            RedTree::Node(node) => Some(StatementLine { span: node.span(), text: source.get_text_in(node.span()).trim().to_string() }),
            _ => None,
        })
        .collect()
}

fn first_word(text: &str) -> String {
    text.trim().split_whitespace().next().unwrap_or_default().to_ascii_lowercase()
}

fn head_word_len(text: &str) -> usize {
    text.trim().split_whitespace().next().map(str::len).unwrap_or(0)
}

fn matches_terminator(text: &str, terminators: &[&str]) -> bool {
    if terminators.is_empty() {
        return false;
    }
    let normalized = text.trim().to_ascii_lowercase();
    terminators.iter().any(|terminator| normalized == *terminator || normalized.starts_with(&format!("{terminator} ")) || normalized.starts_with(&format!("{terminator}(")))
}

fn consumed_end(lines: &[StatementLine], index: usize, fallback: usize) -> usize {
    if index == 0 {
        return fallback;
    }
    lines.get(index - 1).map(|line| line.span.end).unwrap_or(fallback)
}

fn action_text(text: &str) -> String {
    text.trim().strip_prefix(':').unwrap_or(text.trim()).trim_end_matches(';').trim().to_string()
}

fn parse_activity_transition(line: &StatementLine) -> Option<ActivityTransition> {
    let (left, right) = line.text.split_once("-->")?;
    let (target, label) = split_label(right);
    Some(ActivityTransition { from: activity_endpoint(left, true), to: activity_endpoint(target, false), label, span: line.span })
}

fn activity_endpoint(raw: &str, is_left: bool) -> ActivityEndpoint {
    let value = unquote(raw);
    if value.starts_with("(*)") { if is_left { ActivityEndpoint::Start } else { ActivityEndpoint::Stop } } else { ActivityEndpoint::Named(value) }
}

fn parse_swimlane(text: &str) -> (String, Option<String>) {
    let text = text.trim();
    let Some(inner) = text.strip_prefix('|')
    else {
        return (text.to_string(), None);
    };
    let Some(end) = inner.find('|')
    else {
        return (inner.trim().to_string(), None);
    };
    let first = inner[..end].trim();
    let rest = inner[end + 1..].trim();

    if let Some(color) = first.strip_prefix('#') {
        let name = rest.split('|').next().unwrap_or(rest).trim().to_string();
        (name, Some(format!("#{color}")))
    }
    else {
        (first.to_string(), None)
    }
}

fn parenthesized(text: &str) -> Option<String> {
    let start = text.find('(')?;
    let end = text[start..].find(')')?;
    Some(text[start + 1..start + end].trim().to_string())
}

fn parenthesized_after(text: &str, keyword: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let position = lower.find(keyword)? + keyword.len();
    parenthesized(&text[position..])
}

fn parse_state_transition(text: &str) -> Option<StateTransition> {
    let (left, right) = text.split_once("-->").or_else(|| text.split_once("->"))?;
    let (target, label) = split_label(right);
    let from = state_endpoint(left, true);
    let to = state_endpoint(target, false);
    Some(StateTransition { from, to, label })
}

fn state_endpoint(raw: &str, is_left: bool) -> StateEndpoint {
    let value = raw.trim();
    if value == "[*]" { if is_left { StateEndpoint::Start } else { StateEndpoint::End } } else { StateEndpoint::State(unquote(value)) }
}

fn parse_state_header(text: &str, diagnostics: &mut Vec<OakError>, offset: usize) -> (String, Option<String>, StateKind, bool) {
    let rest = text[head_word_len(text)..].trim();
    let has_block = rest.ends_with('{');
    let rest = rest.trim_end_matches('{').trim_end();

    let (display, rest) = if let Some(after) = rest.strip_prefix('"') {
        match after.find('"') {
            Some(end) => (Some(after[..end].to_string()), after[end + 1..].trim()),
            None => {
                diagnostics.push(OakError::syntax_error("unterminated state display name".to_string(), offset, None));
                (None, rest)
            }
        }
    }
    else {
        (None, rest)
    };

    let mut id = String::new();
    let mut tokens = rest.split_whitespace();
    if display.is_some() {
        if tokens.next() == Some("as") {
            if let Some(name) = tokens.next() {
                id = name.to_string();
            }
        }
    }
    else if let Some(name) = tokens.next() {
        if name != "as" && !name.starts_with("<<") {
            id = name.to_string();
        }
        else if name == "as" {
            if let Some(name) = tokens.next() {
                id = name.to_string();
            }
        }
    }

    let kind = if let Some(open) = rest.find("<<") {
        if let Some(close) = rest[open..].find(">>") {
            match rest[open + 2..open + close].trim().to_ascii_lowercase().as_str() {
                "fork" => StateKind::Fork,
                "join" => StateKind::Join,
                "choice" => StateKind::Choice,
                "start" => StateKind::Start,
                "end" => StateKind::End,
                "entrypoint" | "entry" => StateKind::EntryPoint,
                "exitpoint" | "exit" => StateKind::ExitPoint,
                _ => StateKind::Normal,
            }
        }
        else {
            StateKind::Normal
        }
    }
    else {
        StateKind::Normal
    };

    (id, display, kind, has_block)
}

fn parse_named_header(word_len: usize, text: &str) -> (String, Option<String>, Option<String>, bool) {
    let rest = text[word_len..].trim();
    let has_block = rest.ends_with('{');
    let rest = rest.trim_end_matches('{').trim_end();

    let (display, rest) = if let Some(after) = rest.strip_prefix('"') {
        match after.find('"') {
            Some(end) => (Some(after[..end].to_string()), after[end + 1..].trim()),
            None => (None, rest),
        }
    }
    else {
        (None, rest)
    };

    let mut name = String::new();
    let mut alias = None;
    let mut tokens = rest.split_whitespace();

    if display.is_some() {
        if tokens.next() == Some("as") {
            if let Some(value) = tokens.next() {
                alias = Some(value.to_string());
                name = value.to_string();
            }
        }
    }
    else if let Some(first) = tokens.next() {
        if first == "as" {
            if let Some(value) = tokens.next() {
                alias = Some(value.to_string());
                name = value.to_string();
            }
        }
        else {
            name = first.to_string();
            if tokens.next() == Some("as") {
                if let Some(value) = tokens.next() {
                    alias = Some(value.to_string());
                    name = value.to_string();
                }
            }
        }
    }

    (name, display, alias, has_block)
}

fn parse_bracket_header(text: &str) -> (String, Option<String>, Option<String>, bool) {
    let rest = text.trim();
    let (name, remainder) = if let Some(after) = rest.strip_prefix('[') {
        match after.find(']') {
            Some(end) => (after[..end].trim().to_string(), after[end + 1..].trim()),
            None => (rest.to_string(), ""),
        }
    }
    else {
        (rest.to_string(), "")
    };

    let mut tokens = remainder.split_whitespace();
    let alias = if tokens.next() == Some("as") { tokens.next().map(str::to_string) } else { None };
    let resolved = alias.clone().unwrap_or(name.clone());
    (resolved, None, alias, false)
}

fn take_endpoint(rest: &mut &str) -> Option<String> {
    let trimmed = rest.trim_start();
    if trimmed.is_empty() {
        return None;
    }

    if let Some(after) = trimmed.strip_prefix('"') {
        let mut escaped = false;
        let mut end = None;
        for (offset, ch) in after.char_indices() {
            if escaped {
                escaped = false;
                continue;
            }
            if ch == '\\' {
                escaped = true;
            }
            else if ch == '"' {
                end = Some(offset);
                break;
            }
        }
        let Some(end) = end
        else {
            *rest = "";
            return Some(after.to_string());
        };
        *rest = &after[end + 1..];
        return Some(after[..end].to_string());
    }

    if let Some(after) = trimmed.strip_prefix('[') {
        let Some(end) = after.find(']')
        else {
            *rest = "";
            return Some(after.trim().to_string());
        };
        *rest = &after[end + 1..];
        return Some(after[..end].trim().to_string());
    }

    let mut split = trimmed.len();
    for (offset, ch) in trimmed.char_indices() {
        if ch.is_whitespace() {
            split = offset;
            break;
        }
    }
    *rest = &trimmed[split..];
    Some(trimmed[..split].to_string())
}

fn take_multiplicity(rest: &mut &str) -> Option<String> {
    let trimmed = rest.trim_start();
    let after = trimmed.strip_prefix('"')?;
    let end = after.find('"')?;
    *rest = &after[end + 1..];
    Some(after[..end].to_string())
}

fn find_arrow(text: &str) -> Option<&'static str> {
    const ARROWS: &[&str] = &["-->>", "->>", "-->", "->", "<|--", "--|>", "--*", "*--", "--o", "o--", "<..", "..>", "<--", "--", ".."];
    ARROWS.iter().copied().find(|arrow| text.starts_with(arrow))
}

fn split_label(text: &str) -> (&str, Option<String>) {
    let mut in_quotes = false;
    let mut escaped = false;
    for (offset, ch) in text.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => in_quotes = !in_quotes,
            ':' if !in_quotes => {
                let label = text[offset + 1..].trim();
                return (text[..offset].trim(), if label.is_empty() { None } else { Some(label.to_string()) });
            }
            _ => {}
        }
    }
    (text.trim(), None)
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
