use oak_vos::parse;

#[test]
fn parses_vos_schema_through_oak() {
    let root = parse(
        "namespace demo; table User { id: UUID @primary; embedding: vec<3, cosine>; asset: file; }",
    )
    .expect("Oak parses the VOS surface");

    assert!(root.source.contains("table User"));
}

#[test]
fn parses_vos_udf_surface_without_binding_it_to_a_runtime() {
    let root = parse("udf score(a: I64) -> I64 { return a; }").expect("Oak parses the UDF surface");
    assert!(root.source.starts_with("udf"));
}
