mod bare_root;
mod lexer;

#[test]
fn test_von_parsing() {
    use oak_core::{Builder, SourceText, parser::session::ParseSession};
    use oak_von::{VonBuilder, VonLanguage};

    let input = r#"name = "Valkyrie"
        version = 1_0.0
        stable = true
        features = ["speed", "safety", "simplicity",]
        metadata = {
            author = "Yggdrasil"
            tags = ["language", "compiler"]
        }
        null_val = null
        status = Success { code = 200 }
        raw_path = raw"C:\Users\Admin"
    "#;

    let config = VonLanguage::default();
    let builder = VonBuilder::new(&config);
    let source = SourceText::new(input.to_string());
    let mut cache = ParseSession::default();

    let result = builder.build(&source, &[], &mut cache);

    if let Err(e) = &result.result {
        panic!("Parsing failed: {:?}", e)
    }

    let root = result.result.unwrap();
    let val = root.value;

    println!("Parsed VON: {}", val.to_string());

    // Verify fields
    match val {
        oak_von::VonValueNode::Object(obj) => {
            assert_eq!(obj.fields.len(), 8);
            assert_eq!(obj.fields[0].name, "name");
            assert_eq!(obj.fields[1].name, "version");
            assert_eq!(obj.fields[2].name, "stable");
            assert_eq!(obj.fields[3].name, "features");
            assert_eq!(obj.fields[4].name, "metadata");
            assert_eq!(obj.fields[5].name, "null_val");
            assert_eq!(obj.fields[6].name, "status");
            assert_eq!(obj.fields[7].name, "raw_path");

            if let oak_von::VonValueNode::Enum(e) = &obj.fields[6].value {
                assert_eq!(e.variant, "Success");
                assert!(e.payload.is_some())
            }
            else {
                panic!("Expected an enum, got {:?}", obj.fields[6].value)
            }

            if let oak_von::VonValueNode::String(s) = &obj.fields[7].value { assert_eq!(s.value, r#"C:\Users\Admin"#) } else { panic!("Expected a string, got {:?}", obj.fields[7].value) }
        }
        _ => panic!("Expected an object, got {:?}", val),
    }
}
