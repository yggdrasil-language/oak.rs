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
                        }
                    }
                }
            }

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

/// The two supported diagram families.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DiagramKind {
    /// Class diagrams.
    Class,
    /// Sequence diagrams.
    Sequence,
}

fn detect_diagram<S: Source + ?Sized>(state: &ParserState<'_, PlantUmlLanguage, S>) -> DiagramKind {
    let mut index = 0;

    loop {
        match state.peek_kind_at(index) {
            None | Some(PlantUmlTokenType::Eof) => return DiagramKind::Class,
            Some(PlantUmlTokenType::Whitespace) | Some(PlantUmlTokenType::Newline) | Some(PlantUmlTokenType::Comment) => index += 1,
            Some(PlantUmlTokenType::StartMarker) => {
                index += 1;
                while let Some(kind) = state.peek_kind_at(index) {
                    if is_line_end(kind) {
                        break;
                    }
                    index += 1;
                }
            }
            Some(PlantUmlTokenType::EndMarker) => return DiagramKind::Class,
            Some(PlantUmlTokenType::Participant) | Some(PlantUmlTokenType::Actor) | Some(PlantUmlTokenType::Activate) | Some(PlantUmlTokenType::Deactivate) => {
                return DiagramKind::Sequence;
            }
            Some(_) => {
                let mut cursor = index;
                while let Some(kind) = state.peek_kind_at(cursor) {
                    if is_line_end(kind) {
                        break;
                    }
                    if kind == PlantUmlTokenType::Arrow {
                        let Some(token) = state.peek_at(cursor)
                        else {
                            return DiagramKind::Class;
                        };
                        return if is_sequence_arrow(state.source.get_text_in(token.span).as_ref()) { DiagramKind::Sequence } else { DiagramKind::Class };
                    }
                    cursor += 1;
                }
                return DiagramKind::Class;
            }
        }
    }
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

    if !matches!(
        state.peek_kind(),
        Some(PlantUmlTokenType::Plus | PlantUmlTokenType::Minus | PlantUmlTokenType::Hash | PlantUmlTokenType::Tilde)
    ) {
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
