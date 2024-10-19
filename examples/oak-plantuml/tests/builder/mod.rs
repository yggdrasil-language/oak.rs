use oak_core::{Builder, parser::session::ParseSession};
use oak_plantuml::{
    PlantUmlBuilder, PlantUmlLanguage,
    ast::{ActivityBranchKind, ActivityEndpoint, ActivityNode, ClassKeyword, ClassRelationKind, ComponentElement, MemberVisibility, MessageArrow, ParticipantKind, PlantUmlDiagram, PortDirection, StateElement, StateEndpoint, StateKind},
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

#[test]
fn builds_new_activity_syntax_with_decision_and_fork() {
    let source = "@startuml\nstart\n:Collect input;\nif (valid?) then (yes)\n:Store;\nelse (no)\n:Reject;\nendif\nfork\n:Path A;\nfork again\n:Path B;\nend fork\nstop\n@enduml\n";
    let root = build(source);

    let PlantUmlDiagram::Activity(diagram) = &root.diagrams[0]
    else {
        panic!("expected an activity diagram");
    };

    assert_eq!(diagram.nodes.len(), 5, "start, action, decision, fork and stop: {:?}", diagram.nodes);
    assert!(matches!(diagram.nodes[0], ActivityNode::Start));

    let ActivityNode::Action(action) = &diagram.nodes[1]
    else {
        panic!("expected the collect action");
    };
    assert_eq!(action.text, "Collect input");
    assert_eq!(action.span.start, source.find(":Collect input;").unwrap());

    let ActivityNode::Decision(decision) = &diagram.nodes[2]
    else {
        panic!("expected a decision");
    };
    assert_eq!(decision.branches.len(), 2);
    assert_eq!(decision.branches[0].kind, ActivityBranchKind::Then);
    assert_eq!(decision.branches[0].condition.as_deref(), Some("valid?"));
    assert_eq!(decision.branches[0].label.as_deref(), Some("yes"));
    assert!(matches!(decision.branches[0].nodes[0], ActivityNode::Action(_)));
    assert_eq!(decision.branches[1].kind, ActivityBranchKind::Else);
    assert_eq!(decision.branches[1].condition, None);
    assert_eq!(decision.branches[1].label.as_deref(), Some("no"));

    let ActivityNode::Fork(fork) = &diagram.nodes[3]
    else {
        panic!("expected a fork");
    };
    assert_eq!(fork.branches.len(), 2);
    assert_eq!(fork.branches[0].nodes.len(), 1);
    assert_eq!(fork.branches[1].nodes.len(), 1);

    assert!(matches!(diagram.nodes[4], ActivityNode::Stop));
}

#[test]
fn builds_activity_swimlanes() {
    let source = "@startuml\n|Customer|\n:Order;\n|Warehouse|\n:Ship;\n@enduml\n";
    let root = build(source);

    let PlantUmlDiagram::Activity(diagram) = &root.diagrams[0]
    else {
        panic!("expected an activity diagram");
    };

    assert_eq!(diagram.nodes.len(), 2);
    let ActivityNode::Swimlane(customer) = &diagram.nodes[0]
    else {
        panic!("expected the customer swimlane");
    };
    assert_eq!(customer.name, "Customer");
    assert_eq!(customer.color, None);
    assert_eq!(customer.nodes.len(), 1);

    let ActivityNode::Swimlane(warehouse) = &diagram.nodes[1]
    else {
        panic!("expected the warehouse swimlane");
    };
    assert_eq!(warehouse.name, "Warehouse");
    assert_eq!(warehouse.nodes.len(), 1);
}

#[test]
fn builds_legacy_activity_arrows() {
    let source = "@startuml\n(*) --> \"First\"\n\"First\" --> \"Second\" : next\n\"Second\" --> (*)\n@enduml\n";
    let root = build(source);

    let PlantUmlDiagram::Activity(diagram) = &root.diagrams[0]
    else {
        panic!("expected an activity diagram");
    };

    assert_eq!(diagram.nodes.len(), 3);
    let ActivityNode::Transition(first) = &diagram.nodes[0]
    else {
        panic!("expected the first transition");
    };
    assert_eq!(first.from, ActivityEndpoint::Start);
    assert_eq!(first.to, ActivityEndpoint::Named("First".to_string()));
    assert_eq!(first.label, None);

    let ActivityNode::Transition(second) = &diagram.nodes[1]
    else {
        panic!("expected the second transition");
    };
    assert_eq!(second.from, ActivityEndpoint::Named("First".to_string()));
    assert_eq!(second.to, ActivityEndpoint::Named("Second".to_string()));
    assert_eq!(second.label.as_deref(), Some("next"));

    let ActivityNode::Transition(third) = &diagram.nodes[2]
    else {
        panic!("expected the closing transition");
    };
    assert_eq!(third.to, ActivityEndpoint::Stop);
}

#[test]
fn builds_state_diagram_with_composite_regions_and_stereotypes() {
    let source = "@startuml\n[*] --> Idle\nstate Idle\nstate Processing {\n  [*] --> Reading\n  Reading --> Writing\n  --\n  [*] --> Retrying\n}\nProcessing --> Idle : done\nstate Forked <<fork>>\n@enduml\n";
    let root = build(source);

    let PlantUmlDiagram::State(diagram) = &root.diagrams[0]
    else {
        panic!("expected a state diagram");
    };

    assert_eq!(diagram.elements.len(), 5, "start transition, two states, one transition and a stereotype state: {:?}", diagram.elements);

    let StateElement::Transition(start) = &diagram.elements[0]
    else {
        panic!("expected the initial transition");
    };
    assert_eq!(start.from, StateEndpoint::Start);
    assert_eq!(start.to, StateEndpoint::State("Idle".to_string()));

    let StateElement::State(idle) = &diagram.elements[1]
    else {
        panic!("expected the idle state");
    };
    assert_eq!(idle.id, "Idle");
    assert_eq!(idle.kind, StateKind::Normal);
    assert!(idle.regions.is_empty(), "a simple state has no regions");

    let StateElement::State(processing) = &diagram.elements[2]
    else {
        panic!("expected the composite state");
    };
    assert_eq!(processing.id, "Processing");
    assert_eq!(processing.regions.len(), 2);
    assert_eq!(processing.regions[0].elements.len(), 2);
    assert_eq!(processing.regions[1].elements.len(), 1);

    let StateElement::Transition(done) = &diagram.elements[3]
    else {
        panic!("expected the completion transition");
    };
    assert_eq!(done.label.as_deref(), Some("done"));

    let StateElement::State(forked) = &diagram.elements[4]
    else {
        panic!("expected the forked state");
    };
    assert_eq!(forked.id, "Forked");
    assert_eq!(forked.kind, StateKind::Fork);
}

#[test]
fn builds_component_diagram_with_package_ports_and_relations() {
    let source = "@startuml\npackage \"Backend\" as B {\n  component \"API Gateway\" as API\n  component Service\n  port http\n  portout metrics\n  interface Repository\n}\nAPI --> Repository : calls\nB.Service --> DB : reads\n@enduml\n";
    let root = build(source);

    let PlantUmlDiagram::Component(diagram) = &root.diagrams[0]
    else {
        panic!("expected a component diagram");
    };

    assert_eq!(diagram.elements.len(), 1, "the package owns every declaration");
    let ComponentElement::Package(package) = &diagram.elements[0]
    else {
        panic!("expected the backend package");
    };
    assert_eq!(package.alias.as_deref(), Some("B"));
    assert_eq!(package.display.as_deref(), Some("Backend"));
    assert_eq!(package.elements.len(), 5);

    let ComponentElement::Component(api) = &package.elements[0]
    else {
        panic!("expected the api component");
    };
    assert_eq!(api.name, "API");
    assert_eq!(api.display.as_deref(), Some("API Gateway"));

    let ComponentElement::Port(http) = &package.elements[2]
    else {
        panic!("expected the http port");
    };
    assert_eq!(http.name, "http");
    assert_eq!(http.direction, PortDirection::InOut);

    let ComponentElement::Port(metrics) = &package.elements[3]
    else {
        panic!("expected the metrics port");
    };
    assert_eq!(metrics.direction, PortDirection::Out);

    assert!(matches!(package.elements[4], ComponentElement::Interface(_)));

    assert_eq!(diagram.relations.len(), 2);
    assert_eq!(diagram.relations[0].left, "API");
    assert_eq!(diagram.relations[0].right, "Repository");
    assert_eq!(diagram.relations[0].label.as_deref(), Some("calls"));
    assert_eq!(diagram.relations[1].left, "B.Service");
    assert_eq!(diagram.relations[1].right, "DB");
    assert_eq!(diagram.relations[1].label.as_deref(), Some("reads"));
}
