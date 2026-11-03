use oak_core::{Builder, parser::session::ParseSession};
use oak_plantuml::{
    PlantUmlBuilder, PlantUmlLanguage,
    ast::{ClassKeyword, ClassRelationKind, MemberVisibility, MessageArrow, ParticipantKind, PlantUmlDiagram},
};

fn build(source: &str) -> oak_plantuml::PlantUmlRoot {
    let language = PlantUmlLanguage::new();
    let builder = PlantUmlBuilder::new(&language);
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = builder.build(source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    output.result.expect("building should succeed")
}

#[test]
fn builds_class_diagram_with_members_and_relation() {
    let source = "@startuml\nclass \"Display Name\" as Alias {\n+ int x\n- void run()\n# int hidden\n~ Map cache\n}\ninterface Service\nAlias \"1\" --> \"many\" Service : uses\n@enduml\n";
    let root = build(source);

    assert_eq!(root.diagrams.len(), 1);
    let PlantUmlDiagram::Class(diagram) = &root.diagrams[0]
    else {
        panic!("expected a class diagram");
    };

    assert_eq!(diagram.span.start, 0);
    assert_eq!(diagram.span.end, source.len() - 1);
    assert_eq!(diagram.classes.len(), 2);

    let alias = &diagram.classes[0];
    assert_eq!(alias.name, "Alias");
    assert_eq!(alias.display.as_deref(), Some("Display Name"));
    assert_eq!(alias.keyword, ClassKeyword::Class);
    assert_eq!(alias.span.start, source.find("class \"Display Name\"").unwrap());
    assert_eq!(alias.members.len(), 4);
    assert_eq!(alias.members[0].visibility, MemberVisibility::Public);
    assert_eq!(alias.members[0].text, "int x");
    assert!(!alias.members[0].is_method);
    assert_eq!(alias.members[0].span.start, source.find("+ int x").unwrap());
    assert_eq!(alias.members[1].visibility, MemberVisibility::Private);
    assert_eq!(alias.members[1].text, "void run()");
    assert!(alias.members[1].is_method);
    assert_eq!(alias.members[2].visibility, MemberVisibility::Protected);
    assert_eq!(alias.members[2].text, "int hidden");
    assert_eq!(alias.members[3].visibility, MemberVisibility::Package);
    assert_eq!(alias.members[3].text, "Map cache");
    assert_eq!(alias.members[3].span.start, source.find("~ Map cache").unwrap());

    let service = &diagram.classes[1];
    assert_eq!(service.name, "Service");
    assert_eq!(service.keyword, ClassKeyword::Interface);
    assert!(service.members.is_empty());

    assert_eq!(diagram.relations.len(), 1);
    let relation = &diagram.relations[0];
    assert_eq!(relation.left, "Alias");
    assert_eq!(relation.right, "Service");
    assert_eq!(relation.left_multiplicity.as_deref(), Some("1"));
    assert_eq!(relation.right_multiplicity.as_deref(), Some("many"));
    assert_eq!(relation.kind, ClassRelationKind::Association);
    assert!(!relation.marker_at_left);
    assert_eq!(relation.label.as_deref(), Some("uses"));
    assert_eq!(relation.span.start, source.find("Alias \"1\"").unwrap());
}

#[test]
fn normalizes_every_class_relation_arrow() {
    let cases = [
        ("-->", ClassRelationKind::Association, false),
        ("<--", ClassRelationKind::Association, true),
        ("<|--", ClassRelationKind::Inheritance, true),
        ("--|>", ClassRelationKind::Inheritance, false),
        ("*--", ClassRelationKind::Composition, true),
        ("--*", ClassRelationKind::Composition, false),
        ("o--", ClassRelationKind::Aggregation, true),
        ("--o", ClassRelationKind::Aggregation, false),
        ("..>", ClassRelationKind::Dependency, false),
        ("<..", ClassRelationKind::Dependency, true),
        ("..", ClassRelationKind::Link, false),
    ];

    for (arrow, expected_kind, expected_marker_at_left) in cases {
        let source = format!("@startuml\nclass Left\nclass Right\nLeft {arrow} Right\n@enduml\n");
        let root = build(&source);
        let PlantUmlDiagram::Class(diagram) = &root.diagrams[0]
        else {
            panic!("expected a class diagram for `{arrow}`");
        };
        let relation = &diagram.relations[0];
        assert_eq!(relation.kind, expected_kind, "arrow `{arrow}`");
        assert_eq!(relation.marker_at_left, expected_marker_at_left, "arrow `{arrow}`");
    }
}

#[test]
fn builds_sequence_diagram_with_activation_span() {
    let source = "@startuml\nparticipant \"Web UI\" as UI\nactor DB\nUI ->> DB : query\nactivate DB\nDB --> UI : rows\ndeactivate DB\nUI -> UI : render\n@enduml\n";
    let root = build(source);

    assert_eq!(root.diagrams.len(), 1);
    let PlantUmlDiagram::Sequence(diagram) = &root.diagrams[0]
    else {
        panic!("expected a sequence diagram");
    };

    assert_eq!(diagram.span.start, 0);
    assert_eq!(diagram.span.end, source.len() - 1);
    assert_eq!(diagram.participants.len(), 2);

    let ui = &diagram.participants[0];
    assert_eq!(ui.id, "UI");
    assert_eq!(ui.display.as_deref(), Some("Web UI"));
    assert_eq!(ui.kind, ParticipantKind::Participant);
    assert_eq!(ui.span.start, source.find("participant \"Web UI\"").unwrap());

    let db = &diagram.participants[1];
    assert_eq!(db.id, "DB");
    assert_eq!(db.display, None);
    assert_eq!(db.kind, ParticipantKind::Actor);

    assert_eq!(diagram.messages.len(), 3);
    let query = &diagram.messages[0];
    assert_eq!(query.from, "UI");
    assert_eq!(query.to, "DB");
    assert_eq!(query.text.as_deref(), Some("query"));
    assert_eq!(query.arrow, MessageArrow::SolidArrow);
    assert_eq!(query.span.start, source.find("UI ->> DB").unwrap());

    let rows = &diagram.messages[1];
    assert_eq!(rows.arrow, MessageArrow::Dotted);
    assert_eq!(rows.text.as_deref(), Some("rows"));

    let render = &diagram.messages[2];
    assert_eq!(render.from, "UI");
    assert_eq!(render.to, "UI");
    assert_eq!(render.arrow, MessageArrow::Solid);

    assert_eq!(diagram.activations.len(), 1);
    let activation = &diagram.activations[0];
    assert_eq!(activation.participant, "DB");
    assert_eq!(activation.from_message, 1);
    assert_eq!(activation.to_message, 1);
}
