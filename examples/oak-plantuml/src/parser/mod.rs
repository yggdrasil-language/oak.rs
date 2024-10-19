//! Parser that groups PlantUML statements into green-tree elements.

pub mod element_type;

use crate::{
    language::PlantUmlLanguage,
    lexer::{PlantUmlLexer, token_type::PlantUmlTokenType},
    parser::element_type::PlantUmlElementType,
};
use oak_core::{
    Parser, TextEdit,
    parser::{ParseCache, ParseOutput, ParserState, parse_with_lexer},
    source::Source,
};

/// Parser for the PlantUML class and sequence subset.
pub struct PlantUmlParser<'config> {
    /// The PlantUML language configuration.
    _config: &'config PlantUmlLanguage,
}

impl<'config> PlantUmlParser<'config> {
    /// Creates a parser with the given language configuration.
    pub fn new(config: &'config PlantUmlLanguage) -> Self {
        Self { _config: config }
    }
}

impl Parser<PlantUmlLanguage> for PlantUmlParser<'_> {
    fn parse<'a, S: Source + ?Sized>(&self, text: &'a S, edits: &[TextEdit], cache: &'a mut impl ParseCache<PlantUmlLanguage>) -> ParseOutput<'a, PlantUmlLanguage> {
        parse_with_lexer(&PlantUmlLexer::new(self._config), text, edits, cache, |state| {
            let root_checkpoint = state.checkpoint();
            let diagram = detect_diagram(state);
            let mut seen_start = false;
            let mut ended = false;
            let mut active = Vec::new();
            let mut container: Option<((usize, usize), PlantUmlElementType)> = None;

            while state.not_at_end() {
                let Some(kind) = state.peek_kind()
                else {
                    break;
                };

                match kind {
                    PlantUmlTokenType::Eof => {
                        state.bump();
                        break;
                    }
                    PlantUmlTokenType::Newline => state.bump(),
                    PlantUmlTokenType::Comment => parse_comment(state),
                    PlantUmlTokenType::StartMarker => {
                        if seen_start {
                            state.syntax_error("duplicate @startuml marker");
                        }
                        let checkpoint = state.checkpoint();
                        state.bump();
                        skip_to_line_end(state);
                        state.finish_at(checkpoint, PlantUmlElementType::StartMarker);
                        seen_start = true;
                        ended = false;
                    }
                    PlantUmlTokenType::EndMarker => {
                        if !seen_start {
                            state.syntax_error("content after @enduml without an @startuml marker");
                        }
                        if ended {
                            state.syntax_error("duplicate @enduml marker");
                        }
                        close_container(state, &mut container);
                        let checkpoint = state.checkpoint();
                        state.bump();
                        skip_to_line_end(state);
                        state.finish_at(checkpoint, PlantUmlElementType::EndMarker);
                        ended = true;
                    }
                    _ => {
                        if !seen_start {
                            state.syntax_error("content outside @startuml");
                        }
                        if ended {
                            state.syntax_error("content after @enduml");
                        }

                        match diagram {
                            DiagramKind::Class => parse_class_line(state),
                            DiagramKind::Sequence => parse_sequence_line(state, &mut active),
                            DiagramKind::Activity => {
                                ensure_container(state, &mut container, PlantUmlElementType::ActivityDiagram);
                                parse_statement_line(state, PlantUmlElementType::ActivityStatement);
                            }
                            DiagramKind::State => {
                                ensure_container(state, &mut container, PlantUmlElementType::StateDiagram);
                                parse_statement_line(state, PlantUmlElementType::StateStatement);
                            }
                            DiagramKind::Component => {
                                ensure_container(state, &mut container, PlantUmlElementType::ComponentDiagram);
                                parse_statement_line(state, PlantUmlElementType::ComponentStatement);
                            }
                            DiagramKind::Mixed { conflict } => {
                                return Err(state.source.syntax_error("PlantUML source mixes class and sequence declarations, one diagram family per source is required".to_string(), conflict));
                            }
                        }
                    }
                }
            }

            close_container(state, &mut container);

            if !seen_start {
                state.syntax_error("missing @startuml marker");
            }
            else if !ended {
                state.syntax_error("missing @enduml marker");
            }

            Ok(state.finish_at(root_checkpoint, PlantUmlElementType::Root))
        })
    }
}

/// The supported diagram families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagramKind {
    /// Class diagrams.
    Class,
    /// Sequence diagrams.
    Sequence,
    /// Activity diagrams, including the legacy arrow syntax.
    Activity,
    /// State diagrams.
    State,
    /// Component diagrams.
    Component,
    /// Class and sequence declarations are both authored, so no single diagram can represent the source.
    Mixed {
        /// Byte offset of the marker that contradicts the earlier family.
        conflict: usize,
    },
}

/// Decide the diagram family from authored markers.
///
/// Family keywords outrank arrow spelling: a `class` / `enum` / `abstract`
/// declaration names its family explicitly. Line-leading words such as
/// `component`, `state` or `if` name the other families. `interface` alone
/// stays in the class family because component sources always carry
/// `component`, `package`, `port` or a `[bracket]` node.
fn detect_diagram<S: Source + ?Sized>(state: &ParserState<'_, PlantUmlLanguage, S>) -> DiagramKind {
    let mut index = 0;
    let mut at_line_start = true;
    let mut class_marker: Option<usize> = None;
    let mut sequence_marker: Option<usize> = None;
    let mut activity_marker: Option<usize> = None;
    let mut state_marker: Option<usize> = None;
    let mut component_marker: Option<usize> = None;
    let mut sequence_arrow = false;
    let mut class_arrow = false;

    while let Some(kind) = state.peek_kind_at(index) {
        let offset = state.peek_at(index).map(|token| token.span.start).unwrap_or(0);
        match kind {
            PlantUmlTokenType::Eof => break,
            PlantUmlTokenType::Newline => {
                at_line_start = true;
                index += 1;
                continue;
            }
            PlantUmlTokenType::Whitespace | PlantUmlTokenType::Comment => {
                index += 1;
                continue;
            }
            _ => {}
        }

        if at_line_start {
            at_line_start = false;
            let text = state.peek_at(index).map(|token| state.source.get_text_in(token.span).into_owned()).unwrap_or_default();
            let head = text.to_ascii_lowercase();

            if matches!(head.as_str(), "component" | "package" | "port" | "portin" | "portout" | "database" | "node" | "cloud" | "artifact" | "folder" | "frame" | "rectangle" | "queue" | "storage") {
                component_marker.get_or_insert(offset);
            }
            else if kind == PlantUmlTokenType::Text && text.starts_with('[') && text.ends_with(']') {
                if text == "[*]" {
                    state_marker.get_or_insert(offset);
                }
                else {
                    component_marker.get_or_insert(offset);
                }
            }
            else if head == "state" && !next_is_arrow(state, index + 1) {
                state_marker.get_or_insert(offset);
            }
            else if matches!(head.as_str(), "start" | "stop" | "if" | "elseif" | "else" | "endif" | "fork" | "repeat" | "while" | "endwhile" | "partition" | "detach" | "kill") {
                activity_marker.get_or_insert(offset);
            }
            else if kind == PlantUmlTokenType::Colon || kind == PlantUmlTokenType::LeftParen {
                activity_marker.get_or_insert(offset);
            }
            else if kind == PlantUmlTokenType::Text && text.starts_with('|') {
                activity_marker.get_or_insert(offset);
            }
            else if head == "end" && !next_significant_is_left_brace(state, index + 1) {
                activity_marker.get_or_insert(offset);
            }
        }

        match kind {
            PlantUmlTokenType::Class | PlantUmlTokenType::Enum | PlantUmlTokenType::Abstract => {
                class_marker.get_or_insert(offset);
            }
            PlantUmlTokenType::Interface => {
                class_marker.get_or_insert(offset);
            }
            PlantUmlTokenType::Participant | PlantUmlTokenType::Actor | PlantUmlTokenType::Activate | PlantUmlTokenType::Deactivate => {
                sequence_marker.get_or_insert(offset);
            }
            PlantUmlTokenType::Arrow => {
                if let Some(token) = state.peek_at(index) {
                    let arrow = state.source.get_text_in(token.span);
                    if is_sequence_arrow(arrow.as_ref()) {
                        sequence_arrow = true;
                    }
                    if is_class_arrow(arrow.as_ref()) {
                        class_arrow = true;
                    }
                }
            }
            _ => {}
        }
        index += 1;
    }

    if component_marker.is_some() {
        return DiagramKind::Component;
    }
    if state_marker.is_some() {
        return DiagramKind::State;
    }
    if activity_marker.is_some() {
        return DiagramKind::Activity;
    }

    match (class_marker, sequence_marker) {
        (Some(class), Some(sequence)) => DiagramKind::Mixed { conflict: class.max(sequence) },
        (None, Some(_)) => DiagramKind::Sequence,
        (Some(_), None) => DiagramKind::Class,
        (None, None) if sequence_arrow && !class_arrow => DiagramKind::Sequence,
        (None, None) => DiagramKind::Class,
    }
}

fn next_is_arrow<S: Source + ?Sized>(state: &ParserState<'_, PlantUmlLanguage, S>, mut index: usize) -> bool {
    loop {
        match state.peek_kind_at(index) {
            Some(PlantUmlTokenType::Whitespace) => index += 1,
            Some(PlantUmlTokenType::Arrow) => return true,
            _ => return false,
        }
    }
}

fn next_significant_is_left_brace<S: Source + ?Sized>(state: &ParserState<'_, PlantUmlLanguage, S>, mut index: usize) -> bool {
    loop {
        match state.peek_kind_at(index) {
            Some(PlantUmlTokenType::Whitespace) => index += 1,
            Some(PlantUmlTokenType::LeftBrace) => return true,
            _ => return false,
        }
    }
}

fn ensure_container<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>, container: &mut Option<((usize, usize), PlantUmlElementType)>, kind: PlantUmlElementType) {
    if container.is_none() {
        *container = Some((state.checkpoint(), kind));
    }
}

fn close_container<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>, container: &mut Option<((usize, usize), PlantUmlElementType)>) {
    if let Some((checkpoint, kind)) = container.take() {
        state.finish_at(checkpoint, kind);
    }
}

fn parse_statement_line<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>, kind: PlantUmlElementType) {
    let checkpoint = state.checkpoint();
    skip_to_line_end(state);
    state.finish_at(checkpoint, kind);
}

fn parse_class_line<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    match state.peek_kind() {
        Some(PlantUmlTokenType::Class | PlantUmlTokenType::Interface | PlantUmlTokenType::Enum | PlantUmlTokenType::Abstract) => parse_class_declaration(state),
        Some(kind) if is_name_kind(kind) => parse_class_relation(state),
        Some(_) => parse_error_line(state, "unrecognized class diagram statement"),
        None => (),
    }
}

fn parse_sequence_line<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>, active: &mut Vec<String>) {
    match state.peek_kind() {
        Some(PlantUmlTokenType::Participant | PlantUmlTokenType::Actor) => parse_participant(state),
        Some(PlantUmlTokenType::Activate) => {
            let name = parse_activation(state, PlantUmlElementType::Activate);
            if let Some(name) = name {
                if active.contains(&name) {
                    state.syntax_error(format!("duplicate activate for participant `{name}`"));
                }
                else {
                    active.push(name);
                }
            }
        }
        Some(PlantUmlTokenType::Deactivate) => {
            let name = parse_activation(state, PlantUmlElementType::Deactivate);
            if let Some(name) = name {
                if let Some(position) = active.iter().rposition(|active_name| active_name == &name) {
                    active.remove(position);
                }
                else {
                    state.syntax_error(format!("deactivate without matching activate for participant `{name}`"));
                }
            }
        }
        Some(kind) if is_name_kind(kind) => parse_sequence_message(state),
        Some(_) => parse_error_line(state, "unrecognized sequence diagram statement"),
        None => (),
    }
}

fn parse_comment<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();
    state.bump();
    state.finish_at(checkpoint, PlantUmlElementType::Comment);
}

fn parse_class_declaration<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();

    if state.at(PlantUmlTokenType::Abstract) {
        state.bump();
        if state.at(PlantUmlTokenType::Class) {
            state.bump();
        }
        else {
            state.syntax_error("`abstract` must be followed by `class`");
        }
    }
    else {
        state.bump();
    }

    if state.peek_kind().is_some_and(is_name_kind) {
        state.bump();
    }
    else {
        state.syntax_error("expected a class name");
    }

    if state.at(PlantUmlTokenType::As) {
        state.bump();
        if state.peek_kind().is_some_and(is_name_kind) {
            state.bump();
        }
        else {
            state.syntax_error("expected an alias after `as`");
        }
    }

    if state.at(PlantUmlTokenType::Newline) && next_is_left_brace(state) {
        while state.at(PlantUmlTokenType::Newline) {
            state.bump();
        }
    }

    if state.at(PlantUmlTokenType::LeftBrace) {
        parse_member_block(state);
    }

    if !is_line_end_current(state) {
        state.syntax_error("unexpected token after class declaration");
        skip_to_line_end(state);
    }

    state.finish_at(checkpoint, PlantUmlElementType::ClassDeclaration);
}

fn parse_member_block<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();

    if state.at(PlantUmlTokenType::LeftBrace) {
        state.bump();
    }

    while state.not_at_end() {
        match state.peek_kind() {
            Some(PlantUmlTokenType::Newline) => state.bump(),
            Some(PlantUmlTokenType::Comment) => parse_comment(state),
            Some(PlantUmlTokenType::RightBrace) => {
                state.bump();
                break;
            }
            Some(PlantUmlTokenType::Eof) => {
                state.syntax_error("unterminated class member block");
                break;
            }
            Some(_) => parse_class_member(state),
            None => break,
        }
    }

    state.finish_at(checkpoint, PlantUmlElementType::ClassMemberBlock);
}

fn parse_class_member<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();

    if !matches!(state.peek_kind(), Some(PlantUmlTokenType::Plus | PlantUmlTokenType::Minus | PlantUmlTokenType::Hash | PlantUmlTokenType::Tilde)) {
        state.syntax_error("class member must start with `+`, `-`, `#` or `~`");
        skip_to_line_end(state);
        state.finish_at(checkpoint, PlantUmlElementType::Error);
        return;
    }

    state.bump();
    while !is_line_end_current(state) && !state.at(PlantUmlTokenType::RightBrace) {
        state.bump();
    }

    state.finish_at(checkpoint, PlantUmlElementType::ClassMember);
}

fn parse_class_relation<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();
    state.bump();

    if state.at(PlantUmlTokenType::String) {
        state.bump();
    }

    let mut valid = true;
    if state.at(PlantUmlTokenType::Arrow) {
        let arrow = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
        if !is_class_arrow(&arrow) {
            state.syntax_error(format!("unsupported class relation arrow `{arrow}`"));
            valid = false;
        }
        state.bump();
    }
    else {
        state.syntax_error("expected a class relation arrow");
        valid = false;
    }

    if valid {
        if state.at(PlantUmlTokenType::String) {
            state.bump();
        }

        if state.peek_kind().is_some_and(is_name_kind) {
            state.bump();
        }
        else {
            state.syntax_error("expected a right-hand class name");
            valid = false;
        }

        if state.at(PlantUmlTokenType::Colon) {
            state.bump();
            skip_to_line_end(state);
        }
        else if !is_line_end_current(state) {
            state.syntax_error("unexpected token in class relation");
            skip_to_line_end(state);
        }
    }

    if valid {
        state.finish_at(checkpoint, PlantUmlElementType::ClassRelation);
    }
    else {
        skip_to_line_end(state);
        state.finish_at(checkpoint, PlantUmlElementType::Error);
    }
}

fn parse_participant<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();
    state.bump();

    if state.peek_kind().is_some_and(is_name_kind) {
        state.bump();
    }
    else {
        state.syntax_error("expected a participant name");
    }

    if state.at(PlantUmlTokenType::As) {
        state.bump();
        if state.peek_kind().is_some_and(is_name_kind) {
            state.bump();
        }
        else {
            state.syntax_error("expected an alias after `as`");
        }
    }

    if !is_line_end_current(state) {
        state.syntax_error("unexpected token after participant declaration");
        skip_to_line_end(state);
    }

    state.finish_at(checkpoint, PlantUmlElementType::ParticipantDeclaration);
}

fn parse_sequence_message<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    let checkpoint = state.checkpoint();
    state.bump();

    let mut valid = true;
    if state.at(PlantUmlTokenType::Arrow) {
        let arrow = state.peek_text().map(|text| text.into_owned()).unwrap_or_default();
        if !is_sequence_arrow(&arrow) {
            state.syntax_error(format!("unsupported sequence message arrow `{arrow}`"));
            valid = false;
        }
        state.bump();
    }
    else {
        state.syntax_error("expected a sequence message arrow");
        valid = false;
    }

    if valid {
        if state.peek_kind().is_some_and(is_name_kind) {
            state.bump();
        }
        else {
            state.syntax_error("expected a target participant");
            valid = false;
        }

        if state.at(PlantUmlTokenType::Colon) {
            state.bump();
            skip_to_line_end(state);
        }
        else if !is_line_end_current(state) {
            state.syntax_error("unexpected token in sequence message");
            skip_to_line_end(state);
        }
    }

    if valid {
        state.finish_at(checkpoint, PlantUmlElementType::SequenceMessage);
    }
    else {
        skip_to_line_end(state);
        state.finish_at(checkpoint, PlantUmlElementType::Error);
    }
}

fn parse_activation<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>, kind: PlantUmlElementType) -> Option<String> {
    let checkpoint = state.checkpoint();
    state.bump();

    let name = if state.peek_kind().is_some_and(is_name_kind) {
        let text = state.peek_text().map(|text| text.into_owned());
        state.bump();
        text
    }
    else {
        state.syntax_error("expected a participant name");
        None
    };

    if !is_line_end_current(state) {
        state.syntax_error("unexpected token after activation statement");
        skip_to_line_end(state);
    }

    state.finish_at(checkpoint, kind);
    name
}

fn parse_error_line<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>, message: &'static str) {
    if is_line_end_current(state) {
        return;
    }
    state.syntax_error(message);
    let checkpoint = state.checkpoint();
    skip_to_line_end(state);
    state.finish_at(checkpoint, PlantUmlElementType::Error);
}

fn skip_to_line_end<S: Source + ?Sized>(state: &mut ParserState<'_, PlantUmlLanguage, S>) {
    while let Some(kind) = state.peek_kind() {
        if is_line_end(kind) {
            break;
        }
        state.bump();
    }
}

fn is_line_end_current<S: Source + ?Sized>(state: &ParserState<'_, PlantUmlLanguage, S>) -> bool {
    state.peek_kind().is_none_or(is_line_end)
}

fn is_line_end(kind: PlantUmlTokenType) -> bool {
    matches!(kind, PlantUmlTokenType::Newline | PlantUmlTokenType::Eof)
}

fn is_name_kind(kind: PlantUmlTokenType) -> bool {
    matches!(kind, PlantUmlTokenType::Identifier | PlantUmlTokenType::String | PlantUmlTokenType::Text)
}

fn is_sequence_arrow(arrow: &str) -> bool {
    matches!(arrow, "->" | "->>" | "-->" | "-->>")
}

fn is_class_arrow(arrow: &str) -> bool {
    matches!(arrow, "-->" | "<--" | "<|--" | "--|>" | "*--" | "--*" | "o--" | "--o" | "..>" | "<.." | "..")
}

fn next_is_left_brace<S: Source + ?Sized>(state: &ParserState<'_, PlantUmlLanguage, S>) -> bool {
    let mut offset = 0;
    loop {
        match state.peek_kind_at(offset) {
            Some(PlantUmlTokenType::Whitespace) | Some(PlantUmlTokenType::Newline) => offset += 1,
            Some(PlantUmlTokenType::LeftBrace) => return true,
            _ => return false,
        }
    }
}
