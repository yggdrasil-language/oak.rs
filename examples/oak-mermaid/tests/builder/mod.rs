use oak_core::{Builder, SourceText};
use oak_mermaid::{
    MermaidBuilder, MermaidLanguage,
    ast::{EdgeStyle, FlowElement, Flowchart, MermaidDiagram, NodeShape},
};
use oak_testing::{building::BuilderTester, source_from_path};
use std::{path::Path, time::Duration};

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
    flowchart
        .elements
        .iter()
        .find_map(|element| match element {
            FlowElement::Node { id: node_id, text, shape } if node_id == id => Some((text.as_deref(), *shape)),
            _ => None,
        })
        .unwrap_or_else(|| panic!("node `{id}` is missing"))
}

fn find_edge<'a>(flowchart: &'a Flowchart, from: &str, to: &str) -> (Option<&'a str>, EdgeStyle) {
    flowchart
        .elements
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
    assert_eq!(find_node(flowchart, "A"), (None, NodeShape::Box));
    assert_eq!(find_edge(flowchart, "A", "B"), (None, EdgeStyle::Arrow));
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
