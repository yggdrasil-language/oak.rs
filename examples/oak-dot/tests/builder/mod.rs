use oak_core::{Builder, OakError, OakErrorKind, SourceText, parser::session::ParseSession};
use oak_dot::{
    DotBuilder, DotLanguage,
    ast::{AttributeTarget, Compass, EdgeOp, EdgeOperand, GraphType, Statement},
};

/// Builds the typed AST from `source`.
fn build(source: &str) -> (oak_dot::ast::DotRoot, Vec<OakError>) {
    let language = DotLanguage::default();
    let builder = DotBuilder::new(&language);
    let text = SourceText::new(source);
    let mut cache = ParseSession::<DotLanguage>::default();

    let output = builder.build(&text, &[], &mut cache);
    (output.result.expect("the DOT source should build into an AST"), output.diagnostics)
}

/// Returns `true` if a syntax error message mentions `word`.
fn has_syntax_error(diagnostics: &[OakError], word: &str) -> bool {
    diagnostics.iter().any(|error| match error.kind() {
        OakErrorKind::SyntaxError { message, .. } => message.contains(word),
        _ => false,
    })
}

#[test]
fn test_builds_digraph() {
    let source = "strict digraph G { rankdir=LR; A [label=\"Start\", shape=diamond]; A:s -> B:n [label=\"go\"]; }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    assert_eq!(root.graphs.len(), 1);
    let graph = &root.graphs[0];
    assert!(graph.strict);
    assert_eq!(graph.graph_type, GraphType::Digraph);
    assert_eq!(graph.id.as_deref(), Some("G"));
    assert_eq!(&source[graph.span.start..graph.span.end], "strict digraph G { rankdir=LR; A [label=\"Start\", shape=diamond]; A:s -> B:n [label=\"go\"]; }");

    assert_eq!(graph.statements.len(), 3);

    let Statement::Assignment(assignment) = &graph.statements[0]
    else {
        panic!("expected an assignment")
    };
    assert_eq!(assignment.id, "rankdir");
    assert_eq!(assignment.value, "LR");
    assert_eq!(&source[assignment.span.start..assignment.span.end], "rankdir=LR");

    let Statement::Node(node) = &graph.statements[1]
    else {
        panic!("expected a node statement")
    };
    assert_eq!(node.node_id.id, "A");
    assert!(node.node_id.port.is_none());
    let attributes: Vec<_> = node.attributes.iter().map(|attribute| (attribute.name.as_str(), attribute.value.as_deref())).collect();
    assert_eq!(attributes, [("label", Some("Start")), ("shape", Some("diamond"))]);
    assert_eq!(&source[node.span.start..node.span.end], "A [label=\"Start\", shape=diamond]");
    assert_eq!(&source[node.attributes[0].span.start..node.attributes[0].span.end], "label=\"Start\"");

    let Statement::Edge(edge) = &graph.statements[2]
    else {
        panic!("expected an edge statement")
    };
    assert_eq!(&source[edge.span.start..edge.span.end], "A:s -> B:n [label=\"go\"]");

    let EdgeOperand::Node(left) = &edge.left
    else {
        panic!("expected a node operand")
    };
    assert_eq!(left.id, "A");
    let port = left.port.as_ref().expect("the left operand should carry a port");
    assert_eq!(port.id, None);
    assert_eq!(port.compass, Some(Compass::S));
    assert_eq!(&source[port.span.start..port.span.end], ":s");

    assert_eq!(edge.edges.len(), 1);
    let (op, operand) = &edge.edges[0];
    assert_eq!(*op, EdgeOp::Directed);
    let EdgeOperand::Node(right) = operand
    else {
        panic!("expected a node operand")
    };
    assert_eq!(right.id, "B");
    assert_eq!(right.port.as_ref().and_then(|port| port.compass.clone()), Some(Compass::N));

    assert_eq!(edge.attributes.len(), 1);
    assert_eq!(edge.attributes[0].name, "label");
    assert_eq!(edge.attributes[0].value.as_deref(), Some("go"));
}

#[test]
fn test_builds_multi_hop_edges() {
    let source = "digraph { A -> B -> C }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let Statement::Edge(edge) = &root.graphs[0].statements[0]
    else {
        panic!("expected an edge statement")
    };
    assert_eq!(edge.edges.len(), 2);

    let ids: Vec<_> = edge
        .edges
        .iter()
        .map(|(op, operand)| {
            assert_eq!(*op, EdgeOp::Directed);
            match operand {
                EdgeOperand::Node(node) => node.id.clone(),
                EdgeOperand::Subgraph(subgraph) => subgraph.id.clone().unwrap_or_default(),
            }
        })
        .collect();
    assert_eq!(ids, ["B", "C"]);
}

#[test]
fn test_builds_attribute_statements() {
    let source = "graph { graph [bgcolor=gray] node [shape=box] edge [color=red] }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    assert_eq!(root.graphs[0].graph_type, GraphType::Graph);

    let targets: Vec<_> = root.graphs[0]
        .statements
        .iter()
        .map(|statement| match statement {
            Statement::Attribute(attributes) => {
                assert_eq!(attributes.attributes.len(), 1);
                assert!(attributes.attributes[0].value.is_some());
                attributes.target.clone()
            }
            _ => panic!("expected an attribute statement"),
        })
        .collect();
    assert_eq!(targets, [AttributeTarget::Graph, AttributeTarget::Node, AttributeTarget::Edge]);
}

#[test]
fn test_builds_subgraph() {
    let source = "graph { subgraph cluster_x { A -- B } }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let Statement::Subgraph(subgraph) = &root.graphs[0].statements[0]
    else {
        panic!("expected a subgraph statement")
    };
    assert_eq!(subgraph.id.as_deref(), Some("cluster_x"));
    assert_eq!(&source[subgraph.span.start..subgraph.span.end], "subgraph cluster_x { A -- B }");
    assert_eq!(subgraph.statements.len(), 1);

    let Statement::Edge(edge) = &subgraph.statements[0]
    else {
        panic!("expected an edge statement")
    };
    assert_eq!(edge.edges[0].0, EdgeOp::Undirected);
}

#[test]
fn test_builds_anonymous_subgraph_operand() {
    let source = "digraph { { A B } -> C }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let Statement::Edge(edge) = &root.graphs[0].statements[0]
    else {
        panic!("expected an edge statement")
    };
    let EdgeOperand::Subgraph(left) = &edge.left
    else {
        panic!("expected a subgraph operand")
    };
    assert_eq!(left.id, None, "an anonymous subgraph has no identifier");
    assert_eq!(left.statements.len(), 2);

    let EdgeOperand::Node(right) = &edge.edges[0].1
    else {
        panic!("expected a node operand")
    };
    assert_eq!(right.id, "C");
}

#[test]
fn test_unquotes_strings_and_values() {
    let source = "digraph { A [label=\"say \\\"hi\\\"\", style] B [label=box] }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let Statement::Node(first) = &root.graphs[0].statements[0]
    else {
        panic!("expected a node statement")
    };
    assert_eq!(first.attributes[0].value.as_deref(), Some("say \"hi\""));
    assert_eq!(first.attributes[1].name, "style");
    assert_eq!(first.attributes[1].value, None);

    let Statement::Node(second) = &root.graphs[0].statements[1]
    else {
        panic!("expected a node statement")
    };
    assert_eq!(second.attributes[0].value.as_deref(), Some("box"));
}

#[test]
fn test_normalizes_compass_case() {
    let source = "digraph { A -> B:port:ne }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let Statement::Edge(edge) = &root.graphs[0].statements[0]
    else {
        panic!("expected an edge statement")
    };
    let EdgeOperand::Node(right) = &edge.edges[0].1
    else {
        panic!("expected a node operand")
    };
    let port = right.port.as_ref().expect("the operand should carry a port");
    assert_eq!(port.id.as_deref(), Some("port"));
    assert_eq!(port.compass, Some(Compass::NE));
}

#[test]
fn test_single_segment_compass_port() {
    let source = "digraph { a:c }";
    let (root, diagnostics) = build(source);
    assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");

    let Statement::Node(node) = &root.graphs[0].statements[0]
    else {
        panic!("expected a node statement")
    };
    let port = node.node_id.port.as_ref().expect("the node should carry a port");
    assert_eq!(port.id, None);
    assert_eq!(port.compass, Some(Compass::C));
}

#[test]
fn test_reports_unknown_compass_point() {
    let source = "digraph { A -> B:port:sideways }";
    let (root, diagnostics) = build(source);
    assert_eq!(root.graphs.len(), 1);
    assert!(has_syntax_error(&diagnostics, "compass point"), "expected a compass diagnostic, got {diagnostics:?}");

    let Statement::Edge(edge) = &root.graphs[0].statements[0]
    else {
        panic!("expected an edge statement")
    };
    let EdgeOperand::Node(right) = &edge.edges[0].1
    else {
        panic!("expected a node operand")
    };
    let port = right.port.as_ref().expect("the operand should still carry a port");
    assert_eq!(port.id.as_deref(), Some("port"));
    assert_eq!(port.compass, None);
}

#[test]
fn test_recovers_from_malformed_edges() {
    let source = "digraph { A -> ; B -> C }";
    let (root, diagnostics) = build(source);
    assert_eq!(root.graphs.len(), 1);
    assert!(has_syntax_error(&diagnostics, "edge operator"), "expected an edge operator diagnostic, got {diagnostics:?}");

    // The statement after the broken edge still builds.
    assert!(root.graphs[0].statements.iter().any(|statement| matches!(statement, Statement::Edge(edge) if edge.edges.len() == 1)));
}

#[test]
fn test_recovers_from_malformed_ports() {
    let source = "digraph { a:b:c:d }";
    let (root, diagnostics) = build(source);
    assert_eq!(root.graphs.len(), 1);
    assert!(has_syntax_error(&diagnostics, "port"), "expected a port diagnostic, got {diagnostics:?}");

    let Statement::Node(node) = &root.graphs[0].statements[0]
    else {
        panic!("expected a node statement")
    };
    assert_eq!(node.node_id.id, "a");
    let port = node.node_id.port.as_ref().expect("the node should still carry a port");
    assert_eq!(port.id.as_deref(), Some("b"));
}
