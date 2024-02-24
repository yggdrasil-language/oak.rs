use oak_von::language::value::{VonField, VonObject, VonValue};
use oak_von::printer::{PrintOptions, PrintStyle, print_value};

#[test]
fn compact_object_matches_to_source() {
    let value = VonValue::Object(VonObject {
        fields: vec![
            VonField { name: "x".into(), value: VonValue::Number(1.0) },
            VonField { name: "y".into(), value: VonValue::String("hi".into()) },
        ],
    });
    let out = print_value(&value, PrintStyle::Compact, &PrintOptions::default());
    assert_eq!(out, "{x=1,y=\"hi\"}");
}

#[test]
fn indented_object_breaks_lines() {
    let value = VonValue::Object(VonObject {
        fields: vec![
            VonField { name: "x".into(), value: VonValue::Number(1.0) },
            VonField {
                name: "nested".into(),
                value: VonValue::Object(VonObject {
                    fields: vec![VonField { name: "a".into(), value: VonValue::Boolean(true) }],
                }),
            },
        ],
    });
    let out = print_value(&value, PrintStyle::Indented, &PrintOptions::default());
    assert!(out.contains("{\n"));
    assert!(out.contains("    x=1"));
    assert!(out.contains("    nested={\n"));
}
