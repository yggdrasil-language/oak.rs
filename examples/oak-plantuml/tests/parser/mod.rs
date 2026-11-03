use oak_core::{Parser, RedNode, RedTree, parser::session::ParseSession};
use oak_plantuml::{PlantUmlElementType, PlantUmlLanguage, PlantUmlParser};

fn collect_elements(node: RedNode<PlantUmlLanguage>, kinds: &mut Vec<PlantUmlElementType>) {
    for child in node.children() {
        if let RedTree::Node(child) = child {
            kinds.push(child.green.kind);
            collect_elements(child, kinds);
        }
    }
}

fn count(kinds: &[PlantUmlElementType], kind: PlantUmlElementType) -> usize {
    kinds.iter().filter(|candidate| **candidate == kind).count()
}

#[test]
fn groups_class_block_and_relation_elements() {
    let source = "@startuml\nclass Foo {\n+ int x\n- void run()\n}\nFoo \"1\" --> \"many\" Bar : uses\n@enduml\n";
    let language = PlantUmlLanguage::new();
    let parser = PlantUmlParser::new(&language);
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = parser.parse(source, &[], &mut cache);

    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    let root = output.result.expect("parsing should succeed");
    let mut kinds = Vec::new();
    collect_elements(RedNode::new(root, 0), &mut kinds);

    assert_eq!(count(&kinds, PlantUmlElementType::ClassDeclaration), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::ClassMemberBlock), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::ClassMember), 2);
    assert_eq!(count(&kinds, PlantUmlElementType::ClassRelation), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::StartMarker), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::EndMarker), 1);
}

#[test]
fn groups_sequence_elements() {
    let source = "@startuml\nparticipant \"Web UI\" as UI\nactor DB\n' a note\nUI ->> DB : query\nactivate DB\nDB --> UI : rows\ndeactivate DB\nUI -> UI : render\n@enduml\n";
    let language = PlantUmlLanguage::new();
    let parser = PlantUmlParser::new(&language);
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = parser.parse(source, &[], &mut cache);

    assert!(output.diagnostics.is_empty(), "unexpected diagnostics: {:?}", output.diagnostics);
    let root = output.result.expect("parsing should succeed");
    let mut kinds = Vec::new();
    collect_elements(RedNode::new(root, 0), &mut kinds);

    assert_eq!(count(&kinds, PlantUmlElementType::ParticipantDeclaration), 2);
    assert_eq!(count(&kinds, PlantUmlElementType::SequenceMessage), 3);
    assert_eq!(count(&kinds, PlantUmlElementType::Activate), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::Deactivate), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::Comment), 1);
}

#[test]
fn reports_unsupported_statements_instead_of_dropping_them() {
    let source = "@startuml\nskinparam monochrome true\n@enduml\n";
    let language = PlantUmlLanguage::new();
    let parser = PlantUmlParser::new(&language);
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = parser.parse(source, &[], &mut cache);

    assert!(!output.diagnostics.is_empty(), "unsupported statements should produce diagnostics");
    let root = output.result.expect("parsing should still produce a tree");
    let mut kinds = Vec::new();
    collect_elements(RedNode::new(root, 0), &mut kinds);
    assert_eq!(count(&kinds, PlantUmlElementType::Error), 1);
}

#[test]
fn accepts_package_visibility_and_rejects_unknown_markers() {
    let language = PlantUmlLanguage::new();
    let parser = PlantUmlParser::new(&language);

    let supported = "@startuml\nclass C {\n~ Map cache\n}\n@enduml\n";
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = parser.parse(supported, &[], &mut cache);
    assert!(output.diagnostics.is_empty(), "`~` is a supported visibility marker: {:?}", output.diagnostics);
    let root = output.result.expect("parsing should succeed");
    let mut kinds = Vec::new();
    collect_elements(RedNode::new(root, 0), &mut kinds);
    assert_eq!(count(&kinds, PlantUmlElementType::ClassMember), 1);
    assert_eq!(count(&kinds, PlantUmlElementType::Error), 0);

    let unsupported = "@startuml\nclass C {\n! int x\n}\n@enduml\n";
    let mut cache = ParseSession::<PlantUmlLanguage>::default();
    let output = parser.parse(unsupported, &[], &mut cache);
    assert!(!output.diagnostics.is_empty(), "an unknown marker must not be dropped silently");
    let root = output.result.expect("parsing should still produce a tree");
    let mut kinds = Vec::new();
    collect_elements(RedNode::new(root, 0), &mut kinds);
    assert_eq!(count(&kinds, PlantUmlElementType::Error), 1);
}
