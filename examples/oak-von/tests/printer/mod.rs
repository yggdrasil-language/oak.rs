use oak_von::{
    language::value::{VonField, VonObject, VonValue},
    printer::{PrintOptions, PrintStyle, print_value},
};

#[test]
fn compact_object_matches_to_source() {
    let value = VonValue::Object(VonObject { fields: vec![VonField { name: "x".into(), value: VonValue::Number(1.0) }, VonField { name: "y".into(), value: VonValue::String("hi".into()) }] });
    let out = print_value(&value, &PrintOptions::compact());
    assert_eq!(out, "{x=1,y=\"hi\"}");
}

#[test]
fn indented_object_breaks_lines() {
    let value = VonValue::Object(VonObject {
        fields: vec![VonField { name: "x".into(), value: VonValue::Number(1.0) }, VonField { name: "nested_object_with_children".into(), value: VonValue::Object(VonObject { fields: vec![VonField { name: "a".into(), value: VonValue::Boolean(true) }] }) }],
    });
    let out = print_value(&value, &PrintOptions { style: PrintStyle::Indented, indent_width: 4, max_width: 40 });
    assert!(out.contains("{\n"));
    assert!(out.contains("    x=1"));
    assert!(out.contains("nested_object_with_children="), "got: {out:?}");
}
