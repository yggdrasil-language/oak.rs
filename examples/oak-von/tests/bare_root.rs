#[test]
fn bare_root_object_parses() {
    let source = r#"{ name: "legion.tools", version: "workspace" }"#;
    let ast = oak_von::parse(source).expect("parse");
    let value = oak_von::language::value::from_ast(&ast);
    assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("legion.tools"));
}

#[test]
fn newline_before_bare_root_object_parses() {
    let source = "\n{ name: \"legion.tools\" }";
    oak_von::parse(source).expect("parse");
}

#[test]
fn indented_bare_root_object_parses() {
    let source = r#"
    {
        name: "legion.tools",
        version: "workspace",
        build: [
            {
                target: "clr"
            }
        ]
    }
    "#;
    let ast = oak_von::parse(source.trim()).expect("parse");
    let value = oak_von::language::value::from_ast(&ast);
    assert_eq!(value.get("name").and_then(|v| v.as_str()), Some("legion.tools"), "parsed value: {value:?}");
}

#[test]
fn bare_root_object_from_str_deserializes_struct() {
    #[derive(Debug, serde::Deserialize, PartialEq)]
    struct ManifestLike {
        name: String,
        version: String,
    }

    let source = r#"{ name: "legion.tools", version: "workspace" }"#;
    let parsed: ManifestLike = oak_von::from_str(source).expect("from_str");
    assert_eq!(parsed.name, "legion.tools");
    assert_eq!(parsed.version, "workspace");
}
