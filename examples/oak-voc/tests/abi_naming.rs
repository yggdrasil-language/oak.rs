use oak_awsl::is_snake_case;

#[test]
fn snake_case_accepts_valid_names() {
    assert!(is_snake_case("theme"));
    assert!(is_snake_case("theme_change"));
    assert!(is_snake_case("item2"));
}

#[test]
fn snake_case_rejects_camel_and_invalid() {
    assert!(!is_snake_case("themeChange"));
    assert!(!is_snake_case("_theme"));
    assert!(!is_snake_case("theme_"));
    assert!(!is_snake_case(""));
}
