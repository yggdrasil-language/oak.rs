use oak_xpath::{Axis, NodeTest, PathExpr, Predicate, QName, Step, XpathExpr, parse_xpath};

fn path(steps: Vec<Step>) -> PathExpr {
    PathExpr { absolute: false, steps }
}

fn child(name: &str) -> Step {
    Step { axis: Axis::Child, test: NodeTest::Name(QName { prefix: None, local: name.to_string() }), predicates: Vec::new() }
}

#[test]
fn parses_child_path() {
    let expr = parse_xpath("w:body/w:p").expect("parse");
    assert_eq!(
        expr,
        XpathExpr {
            paths: vec![PathExpr {
                absolute: false,
                steps: vec![
                    Step { axis: Axis::Child, test: NodeTest::Name(QName { prefix: Some("w".to_string()), local: "body".to_string() }), predicates: Vec::new() },
                    Step { axis: Axis::Child, test: NodeTest::Name(QName { prefix: Some("w".to_string()), local: "p".to_string() }), predicates: Vec::new() },
                ],
            }],
        }
    );
}

#[test]
fn parses_absolute_and_descendant() {
    let expr = parse_xpath("//w:t").expect("parse");
    assert_eq!(expr.paths.len(), 1);
    assert!(expr.paths[0].absolute);
    assert_eq!(expr.paths[0].steps.len(), 1);
    assert_eq!(expr.paths[0].steps[0].axis, Axis::Descendant);
}

#[test]
fn parses_attribute_predicate() {
    let expr = parse_xpath("w:p[@w14:paraId=\"abc\"]").expect("parse");
    let predicates = &expr.paths[0].steps[0].predicates;
    assert_eq!(predicates, &[Predicate::AttributeEquals(QName { prefix: Some("w14".to_string()), local: "paraId".to_string() }, "abc".to_string(),)]);
}

#[test]
fn parses_position_predicate() {
    let expr = parse_xpath("w:tbl/w:tr[1]").expect("parse");
    assert_eq!(expr.paths[0].steps[1].predicates, vec![Predicate::Position(1)]);
}

#[test]
fn parses_union() {
    let expr = parse_xpath("a|b").expect("parse");
    assert_eq!(expr.paths, vec![path(vec![child("a")]), path(vec![child("b")])]);
}
