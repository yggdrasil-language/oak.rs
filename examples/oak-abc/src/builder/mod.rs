//! Builder turning ABC lines into the typed AST.

use core::range::Range;

use oak_core::{
    Builder, BuilderCache, GreenNode, GreenTree, OakDiagnostics, OakError, Parser, SourceText, TextEdit,
    parser::session::ParseSession,
    source::Source,
};

use crate::{
    ast::{
        AbcAccidental, AbcBarline, AbcChord, AbcClef, AbcDecoration, AbcEvent, AbcFraction, AbcHeader, AbcKey, AbcMeasure,
        AbcMeter, AbcNote, AbcPitch, AbcRest, AbcRoot, AbcSlur, AbcTune, AbcTuplet, AbcUnsupported, AbcVoice,
    },
    language::AbcLanguage,
    parser::{AbcParser, element_type::AbcElementType},
};

/// Builder for the ABC typed AST.
#[derive(Default)]
pub struct AbcBuilder;

impl Builder<AbcLanguage> for AbcBuilder {
    fn build<'a, S: Source + ?Sized>(
        &self,
        source: &S,
        edits: &[TextEdit],
        _cache: &'a mut impl BuilderCache<AbcLanguage>,
    ) -> OakDiagnostics<AbcRoot> {
        let parser = AbcParser;
        let mut cache = ParseSession::<AbcLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut cache);

        match parse_result.result {
            Ok(green_tree) => {
                let text = source.get_text_in((0..source.length()).into()).into_owned();
                let source_text = SourceText::new(text);
                let mut diagnostics = parse_result.diagnostics;
                match self.build_root(green_tree, &source_text) {
                    Ok(root) => OakDiagnostics { result: Ok(root), diagnostics },
                    Err(error) => {
                        diagnostics.push(error.clone());
                        OakDiagnostics { result: Err(error), diagnostics }
                    }
                }
            }
            Err(error) => OakDiagnostics { result: Err(error), diagnostics: parse_result.diagnostics },
        }
    }
}

/// One measure under construction.
#[derive(Default)]
struct MeasureBuild {
    events: Vec<AbcEvent>,
    barline: AbcBarline,
    start: usize,
    end: usize,
    opened: bool,
}

impl MeasureBuild {
    fn into_ast(self) -> AbcMeasure {
        AbcMeasure {
            events: self.events,
            barline: self.barline,
            span: (self.start..self.end.max(self.start)).into(),
        }
    }
}

/// One voice under construction.
#[derive(Default)]
struct VoiceBuild {
    id: Option<String>,
    clef: Option<AbcClef>,
    measures: Vec<MeasureBuild>,
    start: usize,
    end: usize,
}

/// One tune under construction.
#[derive(Default)]
struct TuneBuild {
    index: Option<u32>,
    voices: Vec<VoiceBuild>,
    unsupported: Vec<AbcUnsupported>,
    start: usize,
    end: usize,
}

/// Traversal state for one music line.
#[derive(Default)]
struct LineState {
    accidental: Option<AbcAccidental>,
    slur_start: bool,
    decorations: Vec<AbcDecoration>,
    tuplet: Option<(AbcTuplet, u8)>,
}

impl AbcBuilder {
    fn build_root(&self, green_tree: &GreenNode<AbcLanguage>, source: &SourceText) -> Result<AbcRoot, OakError> {
        if green_tree.kind != AbcElementType::SourceFile {
            return Err(OakError::internal_error("the ABC syntax tree is missing its root element"));
        }

        let mut header = AbcHeader::default();
        let mut header_end = 0usize;
        let mut tunes: Vec<TuneBuild> = Vec::new();
        let mut current: Option<TuneBuild> = None;
        let mut voice_index: Option<usize> = None;
        let mut lyric_targets: Vec<(usize, usize, usize)> = Vec::new();
        let mut document_end = 0usize;
        let mut offset = 0usize;


        for child in green_tree.children {
            let GreenTree::Node(node) = child
            else {
                if let GreenTree::Leaf(leaf) = child {
                    offset += leaf.length as usize;
                }
                continue;
            };
            let start = offset;
            let end = offset + node.byte_length as usize;
            offset = end;
            document_end = end;
            let text = source.get_text_in((start..end).into()).into_owned();
            let line = text.trim_end_matches(['\n', '\r']);
            let body = line.trim();

            if body.is_empty() {
                continue;
            }
            if body.starts_with("%%") {
                let record = unsupported("%%", body, start, start + body.len());
                match current.as_mut() {
                    Some(tune) => tune.unsupported.push(record),
                    None => header.unsupported.push(record),
                }
                continue;
            }
            if body.starts_with('%') {
                continue;
            }
            if body.starts_with("w:") || body.starts_with("W:") {
                let value = body[2..].trim();
                if body.starts_with("W:") {
                    let record = unsupported("text_block", body, start, start + body.len());
                    match current.as_mut() {
                        Some(tune) => tune.unsupported.push(record),
                        None => header.unsupported.push(record),
                    }
                    continue;
                }
                self.attach_lyrics(&mut current, &lyric_targets, value);
                continue;
            }

            if let Some((field, value)) = split_field(body) {
                header_end = end;
                match field {
                    'X' => {
                        if let Some(tune) = current.take() {
                            tunes.push(tune);
                        }
                        let mut tune = TuneBuild { start, end, ..TuneBuild::default() };
                        tune.index = value.trim().parse::<u32>().ok();
                        current = Some(tune);
                        voice_index = Some(0);
                        lyric_targets.clear();
                    }
                    'T' => header.title = non_empty(value),
                    'C' => header.composer = non_empty(value),
                    'Q' => header.tempo_bpm = parse_tempo(value),
                    'M' => header.meter = parse_meter(value),
                    'L' => header.unit_note_length = parse_fraction(value).map(|(numerator, denominator)| AbcFraction::new(numerator, denominator)),
                    'K' => header.key = parse_key(value),
                    'V' => {
                        let (id, clef) = parse_voice(value);
                        let tune = current.get_or_insert_with(|| TuneBuild { start, end, ..TuneBuild::default() });
                        tune.voices.push(VoiceBuild { id, clef, start, end, ..VoiceBuild::default() });
                        voice_index = Some(tune.voices.len() - 1);
                        lyric_targets.clear();
                    }
                    _ => {
                        let record = unsupported("header_field", body, start, start + body.len());
                        match current.as_mut() {
                            Some(tune) => tune.unsupported.push(record),
                            None => header.unsupported.push(record),
                        }
                    }
                }
                continue;
            }

            // A music line.
            let tune = current.get_or_insert_with(|| TuneBuild { start, end, ..TuneBuild::default() });
            if tune.voices.is_empty() {
                tune.voices.push(VoiceBuild { start, end, ..VoiceBuild::default() });
            }
            let index = voice_index.unwrap_or(0).min(tune.voices.len().saturating_sub(1));
            lyric_targets.clear();
            self.parse_music_line(
                body,
                start,
                &header,
                tune,
                index,
                &mut lyric_targets,
                source,
            )?;
            tune.end = end;
        }

        if let Some(tune) = current.take() {
            tunes.push(tune);
        }

        header.span = (0..header_end.max(0)).into();
        let tunes = tunes.into_iter().map(TuneBuild::into_ast).collect::<Vec<_>>();
        Ok(AbcRoot { header, tunes, span: (0..document_end).into() })
    }

    /// Parse one music line into the voice's measures.
    #[allow(clippy::too_many_arguments)]
    fn parse_music_line(
        &self,
        body: &str,
        base: usize,
        header: &AbcHeader,
        tune: &mut TuneBuild,
        voice_index: usize,
        lyric_targets: &mut Vec<(usize, usize, usize)>,
        _source: &SourceText,
    ) -> Result<(), OakError> {
        let unit = header.unit_note_length.unwrap_or_else(|| default_unit(header.meter));
        let bytes = body.as_bytes();
        let mut index = 0usize;
        let mut state = LineState::default();

        while index < bytes.len() {
            let character = body[index..].chars().next().unwrap_or(' ');
            match character {
                ' ' | '\t' => index += 1,
                '{' => {
                    let start = index;
                    while index < bytes.len() && bytes[index] != b'}' {
                        index += 1;
                    }
                    if index < bytes.len() {
                        index += 1;
                    }
                    push_unsupported(tune, voice_index, "grace", &body[start..index], base, start, index);
                }
                '"' => {
                    let start = index;
                    index += 1;
                    while index < bytes.len() && bytes[index] != b'"' {
                        index += 1;
                    }
                    if index < bytes.len() {
                        index += 1;
                    }
                    push_unsupported(tune, voice_index, "annotation", &body[start..index], base, start, index);
                }
                '[' => {
                    if let Some(next) = bytes.get(index + 1).copied()
                        && matches!(next, b'K' | b'M' | b'L' | b'V')
                        && bytes.get(index + 2) == Some(&b':')
                    {
                        let start = index;
                        while index < bytes.len() && bytes[index] != b']' {
                            index += 1;
                        }
                        if index < bytes.len() {
                            index += 1;
                        }
                        push_unsupported(tune, voice_index, "inline_field", &body[start..index], base, start, index);
                        continue;
                    }
                    if bytes.get(index + 1) == Some(&b'|') {
                        close_measure(tune, voice_index, AbcBarline::Regular, base, index, index + 2);
                        index += 2;
                        continue;
                    }
                    // A chord.
                    let chord_start = index;
                    index += 1;
                    let mut notes = Vec::new();
                    while index < bytes.len() && bytes[index] != b']' {
                        let character = body[index..].chars().next().unwrap_or(' ');
                        if character == ' ' || character == '\t' {
                            index += 1;
                            continue;
                        }
                        if let Some((note, consumed)) = parse_note(body, index, base, &mut state, unit)? {
                            notes.push(note);
                            index += consumed;
                        }
                        else {
                            index += character.len_utf8();
                        }
                    }
                    if index < bytes.len() {
                        index += 1;
                    }
                    let (length, consumed) = parse_duration(body, index);
                    index += consumed;
                    let length = scale(unit, length);
                    for note in &mut notes {
                        note.length = length;
                    }
                    let chord = AbcChord {
                        notes,
                        length,
                        span: (base + chord_start..base + index).into(),
                    };
                    let chord_len = chord.notes.len();
                    push_event(tune, voice_index, AbcEvent::Chord(chord), base, chord_start, index);
                    if chord_len > 0 {
                        let (measure_index, event_index) = last_event_position(tune, voice_index);
                        lyric_targets.push((voice_index, measure_index, event_index));
                    }
                }
                '|' => {
                    let mut width = 1usize;
                    let barline = if bytes.get(index + 1) == Some(&b':') {
                        width = 2;
                        AbcBarline::RepeatStart
                    }
                    else if bytes.get(index + 1) == Some(&b']') {
                        width = 2;
                        AbcBarline::Final
                    }
                    else if bytes.get(index + 1) == Some(&b'|') {
                        width = 2;
                        AbcBarline::Double
                    }
                    else {
                        AbcBarline::Regular
                    };
                    close_measure(tune, voice_index, barline, base, index, index + width);
                    index += width;
                }
                ':' => {
                    let width = if bytes.get(index + 1) == Some(&b'|') {
                        2
                    }
                    else if bytes.get(index + 1) == Some(&b':') {
                        2
                    }
                    else {
                        1
                    };
                    let barline = if width == 2 && bytes.get(index + 1) == Some(&b':') {
                        AbcBarline::RepeatBoth
                    }
                    else if width == 2 {
                        AbcBarline::RepeatEnd
                    }
                    else {
                        AbcBarline::Regular
                    };
                    close_measure(tune, voice_index, barline, base, index, index + width);
                    index += width;
                }
                'z' | 'x' => {
                    let start = index;
                    index += 1;
                    let (length, consumed) = parse_duration(body, index);
                    index += consumed;
                    let rest = AbcRest { length: scale(unit, length), span: (base + start..base + index).into() };
                    push_event(tune, voice_index, AbcEvent::Rest(rest), base, start, index);
                }
                '(' => {
                    let start = index;
                    index += 1;
                    if bytes.get(index).is_some_and(u8::is_ascii_digit) {
                        let digits = body[index..].chars().take_while(char::is_ascii_digit).count();
                        let actual = body[index..index + digits].parse::<u8>().unwrap_or(3);
                        index += digits;
                        let mut normal = default_tuplet(actual);
                        if bytes.get(index) == Some(&b':') {
                            index += 1;
                            let count = body[index..].chars().take_while(char::is_ascii_digit).count();
                            if count > 0 {
                                normal = body[index..index + count].parse::<u8>().unwrap_or(normal);
                                index += count;
                            }
                        }
                        state.tuplet = Some((AbcTuplet { actual, normal }, 0));
                        let _ = start;
                    }
                    else {
                        state.slur_start = true;
                    }
                }
                ')' => {
                    index += 1;
                    mark_slur_stop(tune, voice_index);
                }
                '-' => {
                    index += 1;
                    mark_tie(tune, voice_index);
                }
                '.' => {
                    index += 1;
                    state.decorations.push(AbcDecoration::Staccato);
                }
                '!' | '+' => {
                    let delimiter = character;
                    let start = index;
                    index += 1;
                    while index < bytes.len() && body[index..].chars().next() != Some(delimiter) {
                        index += 1;
                    }
                    let name = body[start + 1..index].trim().to_string();
                    if index < bytes.len() {
                        index += 1;
                    }
                    match name.as_str() {
                        "staccato" => state.decorations.push(AbcDecoration::Staccato),
                        "tenuto" => state.decorations.push(AbcDecoration::Tenuto),
                        "accent" | "> " => state.decorations.push(AbcDecoration::Accent),
                        other => state.decorations.push(AbcDecoration::Unknown(other.to_string())),
                    }
                }
                _ => {
                    if let Some((note, consumed)) = parse_note(body, index, base, &mut state, unit)? {
                        let start = index;
                        index += consumed;
                        push_event(tune, voice_index, AbcEvent::Note(note), base, start, index);
                        let (measure_index, event_index) = last_event_position(tune, voice_index);
                        lyric_targets.push((voice_index, measure_index, event_index));
                    }
                    else {
                        index += character.len_utf8();
                    }
                }
            }
        }

        Ok(())
    }

    /// Attach a `w:` lyric line to the notes of the preceding music line.
    fn attach_lyrics(
        &self,
        current: &mut Option<TuneBuild>,
        targets: &[(usize, usize, usize)],
        value: &str,
    ) {
        let Some(tune) = current.as_mut()
        else {
            return;
        };
        let mut syllables = value.split_whitespace();
        for &(voice, measure, event) in targets {
            let Some(syllable) = syllables.next()
            else {
                break;
            };
            if syllable == "*" {
                continue;
            }
            let syllable = syllable.trim_end_matches('-').to_string();
            let Some(voice) = tune.voices.get_mut(voice) else { continue };
            let Some(measure) = voice.measures.get_mut(measure) else { continue };
            let Some(event) = measure.events.get_mut(event) else { continue };
            match event {
                AbcEvent::Note(note) => note.lyric = Some(syllable),
                AbcEvent::Chord(chord) => {
                    if let Some(note) = chord.notes.first_mut() {
                        note.lyric = Some(syllable);
                    }
                }
                _ => {}
            }
        }
    }
}

impl TuneBuild {
    fn into_ast(mut self) -> AbcTune {
        let voices = self
            .voices
            .drain(..)
            .map(|mut voice| {
                if let Some(measure) = voice.measures.last_mut()
                    && !measure.opened
                    && measure.events.is_empty()
                    && voice.measures.len() > 1
                {
                    voice.measures.pop();
                }
                AbcVoice {
                    id: voice.id,
                    clef: voice.clef,
                    measures: voice.measures.drain(..).map(MeasureBuild::into_ast).collect(),
                    span: (voice.start..voice.end.max(voice.start)).into(),
                }
            })
            .collect();
        AbcTune {
            index: self.index,
            voices,
            unsupported: self.unsupported,
            span: (self.start..self.end.max(self.start)).into(),
        }
    }
}

/// Append a refused construct to the tune as an event in the current measure.
fn push_unsupported(
    tune: &mut TuneBuild,
    voice_index: usize,
    construct: &str,
    text: &str,
    base: usize,
    start: usize,
    end: usize,
) {
    let record = unsupported(construct, text, base + start, base + end);
    push_event(tune, voice_index, AbcEvent::Unsupported(record), base, start, end);
}

fn push_event(tune: &mut TuneBuild, voice_index: usize, event: AbcEvent, base: usize, start: usize, end: usize) {
    let voice = match tune.voices.get_mut(voice_index) {
        Some(voice) => voice,
        None => return,
    };
    if voice.measures.is_empty() {
        voice.measures.push(MeasureBuild { start: base + start, end: base + end, ..MeasureBuild::default() });
    }
    let measure = voice.measures.last_mut().expect("measure exists");
    measure.opened = true;
    measure.events.push(event);
    measure.end = base + end;
    voice.end = base + end;
}

/// Close the current measure at a bar line.
fn close_measure(
    tune: &mut TuneBuild,
    voice_index: usize,
    barline: AbcBarline,
    base: usize,
    start: usize,
    end: usize,
) {
    let Some(voice) = tune.voices.get_mut(voice_index)
    else {
        return;
    };
    if let Some(measure) = voice.measures.last_mut()
        && measure.opened
    {
        measure.barline = barline;
        measure.end = base + end;
        measure.opened = false;
        voice.measures.push(MeasureBuild { start: base + end, end: base + end, ..MeasureBuild::default() });
        voice.end = base + end;
        return;
    }
    if voice.measures.is_empty() {
        voice.measures.push(MeasureBuild { start: base + start, end: base + end, barline, ..MeasureBuild::default() });
        voice.measures.push(MeasureBuild { start: base + end, end: base + end, ..MeasureBuild::default() });
        voice.end = base + end;
    }
}

/// The position of the last event in the given voice.
fn last_event_position(tune: &TuneBuild, voice_index: usize) -> (usize, usize) {
    let Some(voice) = tune.voices.get(voice_index)
    else {
        return (0, 0);
    };
    let measure_index = voice.measures.len().saturating_sub(1);
    let event_index = voice.measures.get(measure_index).map(|measure| measure.events.len().saturating_sub(1)).unwrap_or(0);
    (measure_index, event_index)
}

fn mark_tie(tune: &mut TuneBuild, voice_index: usize) {
    let Some(voice) = tune.voices.get_mut(voice_index) else { return };
    let Some(measure) = voice.measures.last_mut() else { return };
    match measure.events.last_mut() {
        Some(AbcEvent::Note(note)) => note.tie = true,
        Some(AbcEvent::Chord(chord)) => {
            if let Some(note) = chord.notes.last_mut() {
                note.tie = true;
            }
        }
        _ => {}
    }
}

fn mark_slur_stop(tune: &mut TuneBuild, voice_index: usize) {
    let Some(voice) = tune.voices.get_mut(voice_index) else { return };
    let Some(measure) = voice.measures.last_mut() else { return };
    match measure.events.last_mut() {
        Some(AbcEvent::Note(note)) => {
            note.slur = if note.slur == AbcSlur::Start { AbcSlur::Both } else { AbcSlur::Stop };
        }
        Some(AbcEvent::Chord(chord)) => {
            if let Some(note) = chord.notes.last_mut() {
                note.slur = if note.slur == AbcSlur::Start { AbcSlur::Both } else { AbcSlur::Stop };
            }
        }
        _ => {}
    }
}

/// Parse one note; returns the note and how many bytes it consumed.
fn parse_note(
    body: &str,
    start: usize,
    base: usize,
    state: &mut LineState,
    unit: AbcFraction,
) -> Result<Option<(AbcNote, usize)>, OakError> {
    let bytes = body.as_bytes();
    let mut index = start;
    let mut accidental = state.accidental.take();
    while index < bytes.len() {
        match bytes[index] {
            b'^' => {
                accidental = Some(if bytes.get(index + 1) == Some(&b'^') {
                    index += 2;
                    AbcAccidental::DoubleSharp
                }
                else {
                    index += 1;
                    AbcAccidental::Sharp
                });
            }
            b'_' => {
                accidental = Some(if bytes.get(index + 1) == Some(&b'_') {
                    index += 2;
                    AbcAccidental::DoubleFlat
                }
                else {
                    index += 1;
                    AbcAccidental::Flat
                });
            }
            b'=' => {
                index += 1;
                accidental = Some(AbcAccidental::Natural);
            }
            _ => break,
        }
    }

    let Some(character) = body[index..].chars().next()
    else {
        return Ok(None);
    };
    if !character.is_ascii_alphabetic() || !matches!(character.to_ascii_uppercase(), 'A'..='G') {
        return Ok(None);
    }
    index += character.len_utf8();
    let mut octave = if character.is_ascii_uppercase() { 4 } else { 5 };
    while index < bytes.len() {
        match bytes[index] {
            b'\'' => {
                octave += 1;
                index += 1;
            }
            b',' => {
                octave -= 1;
                index += 1;
            }
            _ => break,
        }
    }

    let (multiplier, consumed) = parse_duration(body, index);
    index += consumed;
    let mut length = scale(unit, multiplier);

    let mut tuplet = None;
    if let Some((declaration, used)) = state.tuplet.as_mut() {
        length = AbcFraction::new(
            length.numerator * u32::from(declaration.normal.max(1)),
            length.denominator * u32::from(declaration.actual.max(1)),
        );
        tuplet = Some(*declaration);
        *used += 1;
        if *used >= declaration.actual {
            state.tuplet = None;
        }
    }

    let note = AbcNote {
        pitch: AbcPitch { step: character.to_ascii_uppercase(), octave, accidental },
        length,
        tie: false,
        slur: if state.slur_start {
            state.slur_start = false;
            AbcSlur::Start
        }
        else {
            AbcSlur::None
        },
        tuplet,
        decorations: std::mem::take(&mut state.decorations),
        lyric: None,
        span: (base + start..base + index).into(),
    };
    Ok(Some((note, index - start)))
}

/// Parse an ABC duration multiplier starting at `start`.
fn parse_duration(body: &str, start: usize) -> (AbcFraction, usize) {
    let bytes = body.as_bytes();
    let mut index = start;
    let mut numerator = 1u32;
    let mut denominator = 1u32;

    if bytes.get(index).is_some_and(u8::is_ascii_digit) {
        let digits = body[index..].chars().take_while(char::is_ascii_digit).count();
        numerator = body[index..index + digits].parse::<u32>().unwrap_or(1);
        index += digits;
        if bytes.get(index) == Some(&b'/') {
            index += 1;
            let digits = body[index..].chars().take_while(char::is_ascii_digit).count();
            denominator = if digits > 0 {
                let value = body[index..index + digits].parse::<u32>().unwrap_or(1);
                index += digits;
                value
            }
            else {
                2
            };
        }
    }
    else if bytes.get(index) == Some(&b'/') {
        let mut slashes = 0u32;
        while bytes.get(index) == Some(&b'/') {
            slashes += 1;
            index += 1;
        }
        let digits = body[index..].chars().take_while(char::is_ascii_digit).count();
        denominator = if digits > 0 {
            let value = body[index..index + digits].parse::<u32>().unwrap_or(1);
            index += digits;
            (2u32.saturating_pow(slashes.saturating_sub(1))) * value
        }
        else {
            2u32.saturating_pow(slashes)
        };
    }

    let mut dots = 0u32;
    while bytes.get(index) == Some(&b'.') {
        dots += 1;
        index += 1;
    }
    if dots > 0 {
        // A dotted note multiplies by (2 - 2^-dots): 3/2, 7/4, 15/8.
        let scale = 2u32.saturating_pow(dots + 1) - 1;
        numerator *= scale;
        denominator *= 2u32.saturating_pow(dots);
    }

    (AbcFraction::new(numerator, denominator), index - start)
}

/// Multiply a unit length by a multiplier.
fn scale(unit: AbcFraction, multiplier: AbcFraction) -> AbcFraction {
    AbcFraction::new(unit.numerator * multiplier.numerator, unit.denominator * multiplier.denominator)
}

/// The default unit note length implied by the meter.
fn default_unit(meter: Option<AbcMeter>) -> AbcFraction {
    match meter {
        Some(AbcMeter::Explicit { beats, unit }) if u32::from(beats) * 4 < u32::from(unit) * 3 => {
            AbcFraction::new(1, 16)
        }
        _ => AbcFraction::new(1, 8),
    }
}

/// The default `normal` count for a tuplet.
fn default_tuplet(actual: u8) -> u8 {
    match actual {
        2 => 3,
        4 => 3,
        8 => 3,
        other => other.saturating_sub(1).max(1),
    }
}

fn non_empty(value: &str) -> Option<String> {
    let trimmed = value.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

fn unsupported(construct: &str, text: &str, start: usize, end: usize) -> AbcUnsupported {
    AbcUnsupported { construct: construct.to_string(), source: text.trim().to_string(), span: (start..end).into() }
}

/// Split `X:value` into the field letter and value.
fn split_field(body: &str) -> Option<(char, &str)> {
    let mut characters = body.chars();
    let field = characters.next()?;
    if characters.next()? != ':' || !field.is_ascii_alphabetic() {
        return None;
    }
    Some((field, &body[2..]))
}

/// Parse `4/4` into its numerator and denominator.
fn parse_fraction(value: &str) -> Option<(u32, u32)> {
    let trimmed = value.trim();
    let mut parts = trimmed.split('/');
    let numerator = parts.next()?.trim().parse::<u32>().ok()?;
    let denominator = parts.next().unwrap_or("1").trim().parse::<u32>().ok()?;
    Some((numerator, denominator))
}

fn parse_meter(value: &str) -> Option<AbcMeter> {
    let trimmed = value.trim();
    if trimmed.starts_with("C|") {
        return Some(AbcMeter::Cut);
    }
    if trimmed.starts_with('C') && !trimmed.starts_with("C:") {
        return Some(AbcMeter::Common);
    }
    let (beats, unit) = parse_fraction(trimmed)?;
    Some(AbcMeter::Explicit { beats: beats as u8, unit: unit as u8 })
}

fn parse_tempo(value: &str) -> Option<u32> {
    let trimmed = value.trim();
    let after_equals = trimmed.split('=').next_back().unwrap_or(trimmed).trim();
    after_equals.parse::<u32>().ok()
}

fn parse_key(value: &str) -> Option<AbcKey> {
    let trimmed = value.trim();
    let mut characters = trimmed.chars();
    let tonic = characters.next()?.to_ascii_uppercase();
    if !matches!(tonic, 'A'..='G') {
        return None;
    }
    let mut rest = &trimmed[tonic.len_utf8()..];
    let accidental = if rest.starts_with("^^") {
        rest = &rest[2..];
        Some(AbcAccidental::DoubleSharp)
    }
    else if rest.starts_with("__") {
        rest = &rest[2..];
        Some(AbcAccidental::DoubleFlat)
    }
    else if rest.starts_with('^') {
        rest = &rest[1..];
        Some(AbcAccidental::Sharp)
    }
    else if rest.starts_with('_') {
        rest = &rest[1..];
        Some(AbcAccidental::Flat)
    }
    else if rest.starts_with('=') {
        rest = &rest[1..];
        Some(AbcAccidental::Natural)
    }
    else {
        None
    };
    let mode = rest.trim().to_lowercase();
    Some(AbcKey { tonic, accidental, mode })
}

fn parse_voice(value: &str) -> (Option<String>, Option<AbcClef>) {
    let trimmed = value.trim();
    let mut id = None;
    let mut clef = None;
    for (index, part) in trimmed.split_whitespace().enumerate() {
        if index == 0 && !part.contains('=') {
            id = Some(part.to_string());
        }
        else if let Some(name) = part.strip_prefix("clef=") {
            clef = Some(match name.trim() {
                "treble" => AbcClef::Treble,
                "bass" => AbcClef::Bass,
                "alto" => AbcClef::Alto,
                other => AbcClef::Other(other.to_string()),
            });
        }
    }
    (id, clef)
}
