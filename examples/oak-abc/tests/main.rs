//! ABC parsing tests.

use oak_abc::{
    ast::{AbcAccidental, AbcBarline, AbcDecoration, AbcEvent, AbcFraction, AbcMeter, AbcSlur},
    parse_abc,
};

const SMOKE: &str = "X:1\nT:Smoke\nM:4/4\nL:1/4\nK:C\nCDEF|\n";

#[test]
fn header_fields_and_one_measure_are_parsed() {
    let root = parse_abc(SMOKE).expect("smoke tune parses");
    assert_eq!(root.header.title.as_deref(), Some("Smoke"));
    assert_eq!(root.header.meter, Some(AbcMeter::Explicit { beats: 4, unit: 4 }));
    assert_eq!(root.header.unit_note_length, Some(AbcFraction::new(1, 4)));
    assert_eq!(root.header.key.as_ref().map(|key| key.tonic), Some('C'));

    assert_eq!(root.tunes.len(), 1);
    let tune = &root.tunes[0];
    assert_eq!(tune.index, Some(1));
    assert_eq!(tune.voices.len(), 1);
    let measures = &tune.voices[0].measures;
    assert_eq!(measures.len(), 1, "the trailing bar closes the only measure");
    assert_eq!(measures[0].events.len(), 4);
    assert_eq!(measures[0].barline, AbcBarline::Regular);
}

#[test]
fn note_length_is_a_fraction_of_a_whole_note() {
    let root = parse_abc(SMOKE).expect("smoke tune parses");
    let AbcEvent::Note(note) = &root.tunes[0].voices[0].measures[0].events[0]
    else {
        panic!("first event is a note");
    };
    assert_eq!(note.pitch.step, 'C');
    assert_eq!(note.pitch.octave, 4);
    assert_eq!(note.length, AbcFraction::new(1, 4));
}

#[test]
fn octave_marks_and_accidentals_are_recorded() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\n^C, D' _E =F|\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;

    let AbcEvent::Note(first) = &events[0] else { panic!("note") };
    assert_eq!(first.pitch.accidental, Some(AbcAccidental::Sharp));
    assert_eq!(first.pitch.octave, 3, "a trailing comma drops an octave");

    let AbcEvent::Note(second) = &events[1] else { panic!("note") };
    assert_eq!(second.pitch.octave, 5, "an apostrophe raises an octave");

    let AbcEvent::Note(third) = &events[2] else { panic!("note") };
    assert_eq!(third.pitch.accidental, Some(AbcAccidental::Flat));

    let AbcEvent::Note(fourth) = &events[3] else { panic!("note") };
    assert_eq!(fourth.pitch.accidental, Some(AbcAccidental::Natural));
}

#[test]
fn durations_include_fractions_and_dots() {
    let source = "X:1\nM:4/4\nL:1/8\nK:C\nC2 C/2 C3/2 C.|\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;
    let lengths: Vec<AbcFraction> = events
        .iter()
        .map(|event| match event {
            AbcEvent::Note(note) => note.length,
            _ => panic!("note"),
        })
        .collect();
    assert_eq!(lengths[0], AbcFraction::new(1, 4), "`C2` doubles the unit");
    assert_eq!(lengths[1], AbcFraction::new(1, 16), "`C/2` halves the unit");
    assert_eq!(lengths[2], AbcFraction::new(3, 16));
    assert_eq!(lengths[3], AbcFraction::new(3, 16), "a dot adds half again");
}

#[test]
fn chords_share_one_length() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\n[CEG]2|\n";
    let root = parse_abc(source).expect("parses");
    let AbcEvent::Chord(chord) = &root.tunes[0].voices[0].measures[0].events[0]
    else {
        panic!("chord");
    };
    assert_eq!(chord.notes.len(), 3);
    assert_eq!(chord.length, AbcFraction::new(1, 2));
    assert!(chord.notes.iter().all(|note| note.length == AbcFraction::new(1, 2)));
}

#[test]
fn tie_slur_and_decorations_attach_to_the_note() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\n(C-D) .E|\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;

    let AbcEvent::Note(first) = &events[0] else { panic!("note") };
    assert_eq!(first.slur, AbcSlur::Start);
    assert!(first.tie, "the hyphen marks a tie");

    let AbcEvent::Note(second) = &events[1] else { panic!("note") };
    assert_eq!(second.slur, AbcSlur::Stop);

    let AbcEvent::Note(third) = &events[2] else { panic!("note") };
    assert!(third.decorations.contains(&AbcDecoration::Staccato));
}

#[test]
fn tuplet_compresses_the_next_notes() {
    let source = "X:1\nM:4/4\nL:1/8\nK:C\n(3CDE F2|\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;
    let AbcEvent::Note(first) = &events[0] else { panic!("note") };
    assert_eq!(first.tuplet.map(|tuplet| tuplet.actual), Some(3));
    assert_eq!(first.length, AbcFraction::new(1, 12), "3:2 over an eighth note");

    let AbcEvent::Note(fourth) = &events[3] else { panic!("note") };
    assert!(fourth.tuplet.is_none(), "the tuplet covers only three notes");
}

#[test]
fn repeat_barlines_are_distinguished() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\nC D|:E F:|G A::B c|]\n";
    let root = parse_abc(source).expect("parses");
    let barlines: Vec<AbcBarline> = root.tunes[0].voices[0].measures.iter().map(|measure| measure.barline).collect();
    assert!(barlines.contains(&AbcBarline::RepeatStart));
    assert!(barlines.contains(&AbcBarline::RepeatEnd));
    assert!(barlines.contains(&AbcBarline::RepeatBoth));
    assert_eq!(barlines.last(), Some(&AbcBarline::Final));
}

#[test]
fn lyrics_align_with_the_preceding_music_line() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\nC D E F|\nw:do re mi fa\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;
    let lyrics: Vec<&str> = events
        .iter()
        .map(|event| match event {
            AbcEvent::Note(note) => note.lyric.as_deref().unwrap_or(""),
            _ => "",
        })
        .collect();
    assert_eq!(lyrics, vec!["do", "re", "mi", "fa"]);
}

#[test]
fn directives_groce_and_inline_fields_are_refused_not_dropped() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\n%%MIDI program 1\n{AG}C [K:G]D|\n\"Am\"E F|\n";
    let root = parse_abc(source).expect("parses with refusals");
    let tune = &root.tunes[0];
    assert!(tune.unsupported.iter().any(|record| record.construct == "%%"), "{:#?}", tune.unsupported);

    let refused: Vec<&str> = tune.voices[0]
        .measures
        .iter()
        .flat_map(|measure| measure.events.iter())
        .filter_map(|event| match event {
            AbcEvent::Unsupported(record) => Some(record.construct.as_str()),
            _ => None,
        })
        .collect();
    assert!(refused.contains(&"grace"), "grace notes are refused: {refused:?}");
    assert!(refused.contains(&"inline_field"), "inline field changes are refused: {refused:?}");
    assert!(refused.contains(&"annotation"), "chord symbols are refused: {refused:?}");
}

#[test]
fn two_tunes_split_on_the_reference_field() {
    let source = "X:1\nT:First\nM:4/4\nL:1/4\nK:C\nC D|\nX:2\nT:Second\nM:3/4\nL:1/4\nK:G\nG A B|\n";
    let root = parse_abc(source).expect("parses");
    assert_eq!(root.tunes.len(), 2);
    assert_eq!(root.tunes[0].index, Some(1));
    assert_eq!(root.tunes[1].index, Some(2));
    assert_eq!(root.tunes[1].voices[0].measures[0].barline, AbcBarline::Regular);
}

#[test]
fn voices_and_their_clefs_are_kept_separate() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\nV:1 clef=treble\nC D|\nV:2 clef=bass\nC, D,|\n";
    let root = parse_abc(source).expect("parses");
    let tune = &root.tunes[0];
    assert_eq!(tune.voices.len(), 2);
    assert_eq!(tune.voices[0].id.as_deref(), Some("1"));
    assert_eq!(tune.voices[1].id.as_deref(), Some("2"));
    assert_eq!(tune.voices[1].clef.as_ref().map(|clef| format!("{clef:?}")), Some("Bass".to_string()));
    assert_eq!(tune.voices[0].measures[0].events.len(), 2);
    assert_eq!(tune.voices[1].measures[0].events.len(), 2);
}

#[test]
fn rests_are_kept_with_their_length() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\nC z2 E|\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;
    assert!(matches!(events[1], AbcEvent::Rest(_)));
    if let AbcEvent::Rest(rest) = &events[1] {
        assert_eq!(rest.length, AbcFraction::new(1, 2));
    }
}

#[test]
fn note_spans_select_the_authored_text() {
    let source = "X:1\nM:4/4\nL:1/4\nK:C\n^C,D'|\n";
    let root = parse_abc(source).expect("parses");
    let events = &root.tunes[0].voices[0].measures[0].events;
    let AbcEvent::Note(first) = &events[0] else { panic!("note") };
    let text = &source[first.span.start..first.span.end];
    assert_eq!(text, "^C,");
}
