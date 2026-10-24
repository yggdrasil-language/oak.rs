use oak_core::{Builder, SourceText};
use oak_mermaid::{
    MermaidBuilder, MermaidLanguage,
    ast::{ClassDiagram, EdgeStyle, FlowElement, Flowchart, MermaidDiagram, MermaidRoot, MessageArrow, NodeShape, ParticipantKind, SequenceDiagram},
};
use oak_testing::{building::BuilderTester, source_from_path};
use std::{path::Path, time::Duration};

fn build_root(source_text: &str) -> MermaidRoot {
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let source = SourceText::new(source_text);
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = builder.build(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    output.result.expect("source should build")
}

fn sequence(root: &MermaidRoot, index: usize) -> &SequenceDiagram {
    match &root.diagrams[index] {
        MermaidDiagram::Sequence(sequence) => sequence,
        other => panic!("expected a sequence diagram, got {other:?}"),
    }
}

fn class_diagram(root: &MermaidRoot, index: usize) -> &ClassDiagram {
    match &root.diagrams[index] {
        MermaidDiagram::Class(class) => class,
        other => panic!("expected a class diagram, got {other:?}"),
    }
}

fn fixture_flowchart() -> Flowchart {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = source_from_path(&here.join("tests/builder/basic.mmd")).expect("fixture should be readable");
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = builder.build(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    let root = output.result.expect("flowchart fixture should build");
    assert_eq!(root.diagrams.len(), 1, "expected exactly one diagram");

    match root.diagrams.into_iter().next().unwrap() {
        MermaidDiagram::Flowchart(flowchart) => flowchart,
        other => panic!("expected a flowchart, got {other:?}"),
    }
}

fn find_node<'a>(flowchart: &'a Flowchart, id: &str) -> (Option<&'a str>, NodeShape) {
    find_node_in(&flowchart.elements, id)
}

fn find_node_in<'a>(elements: &'a [FlowElement], id: &str) -> (Option<&'a str>, NodeShape) {
    elements
        .iter()
        .find_map(|element| match element {
            FlowElement::Node { id: node_id, text, shape } if node_id == id => Some((text.as_deref(), *shape)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("node `{id}` is missing"))
}

fn find_edge<'a>(flowchart: &'a Flowchart, from: &str, to: &str) -> (Option<&'a str>, EdgeStyle) {
    find_edge_in(&flowchart.elements, from, to)
}

fn find_edge_in<'a>(elements: &'a [FlowElement], from: &str, to: &str) -> (Option<&'a str>, EdgeStyle) {
    elements
        .iter()
        .find_map(|element| match element {
            FlowElement::Edge { from: source, to: target, text, style } if source == from && target == to => Some((text.as_deref(), *style)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("edge `{from} -> {to}` is missing"))
}

#[test]
fn test_mermaid_builder_fixture() -> Result<(), oak_core::OakError> {
    let here = Path::new(env!("CARGO_MANIFEST_DIR"));
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let tester = BuilderTester::new(here.join("tests/builder")).with_extension("mmd").with_timeout(Duration::from_secs(5));
    tester.run_tests::<MermaidLanguage, MermaidBuilder>(&builder)
}

#[test]
fn test_flowchart_ast() {
    let flowchart = fixture_flowchart();

    assert_eq!(flowchart.orientation, "TD");

    let node_count = flowchart.elements.iter().filter(|element| matches!(element, FlowElement::Node { .. })).count();
    let edge_count = flowchart.elements.iter().filter(|element| matches!(element, FlowElement::Edge { .. })).count();
    assert_eq!(node_count, 14, "unexpected node count");
    assert_eq!(edge_count, 14, "unexpected edge count");

    assert_eq!(find_node(&flowchart, "A"), (Some("Start"), NodeShape::Box));
    assert_eq!(find_node(&flowchart, "B"), (Some("Decision"), NodeShape::Rhombus));
    assert_eq!(find_node(&flowchart, "C"), (Some("End"), NodeShape::Box));
    assert_eq!(find_node(&flowchart, "D"), (None, NodeShape::Box));
    assert_eq!(find_node(&flowchart, "E"), (Some("Round"), NodeShape::Round));
    assert_eq!(find_node(&flowchart, "F"), (Some("Stadium"), NodeShape::Stadium));
    assert_eq!(find_node(&flowchart, "G"), (Some("Subroutine"), NodeShape::Subroutine));
    assert_eq!(find_node(&flowchart, "H"), (Some("Cylinder"), NodeShape::Cylinder));
    assert_eq!(find_node(&flowchart, "I"), (Some("Circle"), NodeShape::Circle));
    assert_eq!(find_node(&flowchart, "J"), (Some("Rhombus"), NodeShape::Rhombus));

    assert_eq!(find_edge(&flowchart, "A", "B"), (None, EdgeStyle::Arrow));
    assert_eq!(find_edge(&flowchart, "B", "C"), (Some("yes"), EdgeStyle::Arrow));
    assert_eq!(find_edge(&flowchart, "C", "A"), (None, EdgeStyle::DottedArrow));
    assert_eq!(find_edge(&flowchart, "J", "K"), (None, EdgeStyle::Line));
    assert_eq!(find_edge(&flowchart, "K", "L"), (None, EdgeStyle::ThickArrow));
    assert_eq!(find_edge(&flowchart, "L", "M"), (Some("Yes"), EdgeStyle::Arrow));
    assert_eq!(find_edge(&flowchart, "M", "N"), (None, EdgeStyle::Arrow));
}

#[test]
fn test_unsupported_statements_are_skipped() {
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let source = SourceText::new("graph LR\n    subgraph One\n    A --> B\n    end\n    direction TB\n    style A fill:#f9f\n    classDef big fill:#f96\n");
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = builder.build(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    let root = output.result.expect("flowchart should build");
    assert_eq!(root.diagrams.len(), 1);

    let MermaidDiagram::Flowchart(flowchart) = &root.diagrams[0]
    else {
        panic!("expected a flowchart");
    };
    assert_eq!(flowchart.orientation, "LR");
    assert_eq!(flowchart.elements.len(), 1, "subgraph should be the only top-level element");

    let FlowElement::Subgraph(subgraph) = &flowchart.elements[0]
    else {
        panic!("expected a subgraph");
    };
    assert_eq!(subgraph.id, None);
    assert_eq!(subgraph.title, "One");
    assert_eq!(find_node_in(&subgraph.elements, "A"), (None, NodeShape::Box));
    assert_eq!(find_edge_in(&subgraph.elements, "A", "B"), (None, EdgeStyle::Arrow));
}

#[test]
fn test_nested_subgraphs_keep_titles_and_ids() {
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let source = SourceText::new("graph TD\n    subgraph One\n        A --> B\n        subgraph two[Inner]\n            C\n        end\n    end\n    B --> D\n");
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = builder.build(&source, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    let root = output.result.expect("flowchart should build");

    let MermaidDiagram::Flowchart(flowchart) = &root.diagrams[0]
    else {
        panic!("expected a flowchart");
    };

    let FlowElement::Subgraph(outer) = &flowchart.elements[0]
    else {
        panic!("expected the outer subgraph");
    };
    assert_eq!(outer.id, None);
    assert_eq!(outer.title, "One");
    assert_eq!(find_edge_in(&outer.elements, "A", "B"), (None, EdgeStyle::Arrow));

    let nested = outer
        .elements
        .iter()
        .find_map(|element| match element {
            FlowElement::Subgraph(subgraph) => Some(subgraph),
            _ => None,
        })
        .expect("nested subgraph is missing");
    assert_eq!(nested.id.as_deref(), Some("two"));
    assert_eq!(nested.title, "Inner");
    assert_eq!(find_node_in(&nested.elements, "C"), (None, NodeShape::Box));

    assert_eq!(find_edge(flowchart, "B", "D"), (None, EdgeStyle::Arrow));
}

#[test]
fn test_unterminated_subgraph_still_builds() {
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let source = SourceText::new("graph TD\n    subgraph Open\n        A --> B\n");
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = builder.build(&source, &[], &mut cache);
    let root = output.result.expect("flowchart should build");
    let MermaidDiagram::Flowchart(flowchart) = &root.diagrams[0]
    else {
        panic!("expected a flowchart");
    };

    let FlowElement::Subgraph(subgraph) = &flowchart.elements[0]
    else {
        panic!("expected a subgraph");
    };
    assert_eq!(subgraph.title, "Open");
    assert_eq!(find_edge_in(&subgraph.elements, "A", "B"), (None, EdgeStyle::Arrow));
}

#[test]
fn test_empty_source_builds_no_diagrams() {
    let language = MermaidLanguage::default();
    let builder = MermaidBuilder::new(&language);
    let source = SourceText::new("");
    let mut cache = oak_core::parser::ParseSession::<MermaidLanguage>::default();

    let output = builder.build(&source, &[], &mut cache);
    let root = output.result.expect("empty document should build");
    assert!(root.diagrams.is_empty());
    assert_eq!(root.span.start, 0);
    assert_eq!(root.span.end, 0);
}

#[test]
fn test_sequence_diagram_ast() {
    let root = build_root("sequenceDiagram\n    participant A as Alice\n    actor B\n    A->>B: ping\n    B-->A: pong\n    A->B: notify\n    B->>A: done\n");
    assert_eq!(root.diagrams.len(), 1);

    let sequence = sequence(&root, 0);
    assert_eq!(sequence.participants.len(), 2);
    assert_eq!(sequence.participants[0].id, "A");
    assert_eq!(sequence.participants[0].display.as_deref(), Some("Alice"));
    assert_eq!(sequence.participants[0].kind, ParticipantKind::Participant);
    assert_eq!(sequence.participants[1].id, "B");
    assert_eq!(sequence.participants[1].display, None);
    assert_eq!(sequence.participants[1].kind, ParticipantKind::Actor);

    assert_eq!(sequence.messages.len(), 4);
    let message = &sequence.messages[0];
    assert_eq!((message.from.as_str(), message.to.as_str(), message.text.as_str()), ("A", "B", "ping"));
    assert_eq!(message.arrow, MessageArrow::SolidArrow);
    assert_eq!(sequence.messages[1].arrow, MessageArrow::Dotted);
    assert_eq!(sequence.messages[2].arrow, MessageArrow::Solid);
    assert_eq!(sequence.messages[3].arrow, MessageArrow::SolidArrow);
    assert_eq!(sequence.messages[3].text, "done");
}

#[test]
fn test_class_diagram_ast() {
    let root = build_root("classDiagram\n    class Animal {\n        +int age\n        +isMammal()\n    }\n    class Duck {\n        +String beakColor\n        +swim()\n    }\n    Animal <|-- Duck : is a\n    Duck *-- Wing : has\n    Duck ..> Water\n");
    assert_eq!(root.diagrams.len(), 1);

    let class = class_diagram(&root, 0);
    assert_eq!(class.classes.len(), 2);
    assert_eq!(class.classes[0].name, "Animal");
    assert_eq!(class.classes[0].members.len(), 2);
    assert_eq!(class.classes[0].members[0].name, "+int age");
    assert_eq!(class.classes[0].members[0].kind, oak_uml::model::MemberKind::Field);
    assert_eq!(class.classes[0].members[1].name, "+isMammal()");
    assert_eq!(class.classes[0].members[1].kind, oak_uml::model::MemberKind::Method);
    assert_eq!(class.classes[1].name, "Duck");
    assert_eq!(class.classes[1].members.len(), 2);

    assert_eq!(class.relations.len(), 3);
    assert_eq!(class.relations[0].source, "Animal");
    assert_eq!(class.relations[0].target, "Duck");
    assert_eq!(class.relations[0].kind, oak_uml::model::RelationKind::Inheritance);
    assert_eq!(class.relations[0].label.as_deref(), Some("is a"));
    assert_eq!(class.relations[1].kind, oak_uml::model::RelationKind::Composition);
    assert_eq!(class.relations[1].label.as_deref(), Some("has"));
    assert_eq!(class.relations[2].kind, oak_uml::model::RelationKind::Dependency);
    assert_eq!(class.relations[2].label, None);
}
