#[test]
fn decodes_javascript_string_escapes() {
    assert_eq!(oak_typescript::decode_string_literal_text(r#"'sku\u002d1'"#).unwrap(), "sku-1");
    assert_eq!(oak_typescript::decode_string_literal_text(r#"'a\'b'"#).unwrap(), "a'b");
    assert_eq!(oak_typescript::decode_string_literal_text(r#"'\x41\u{1f600}\ud83d\ude00'"#).unwrap(), "A😀😀");
    assert_eq!(oak_typescript::decode_string_literal_text("'a\\\r\nb'").unwrap(), "ab");
}

#[test]
fn rejects_invalid_string_escapes() {
    for source in [r#"'\u12'"#, r#"'\xZZ'"#, r#"'\u{110000}'"#, r#"'\ud800'"#] {
        assert!(oak_typescript::decode_string_literal_text(source).is_err(), "{source}");
    }
}
