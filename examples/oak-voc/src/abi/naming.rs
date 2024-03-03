//! AWSL ABI naming rules.

/// Normalize a template `:prop` key to snake_case property name.
pub fn normalize_prop_name(name: &str) -> String {
    name.trim_start_matches(':').to_string()
}

/// Normalize a template `@event` / `on:` key to snake_case event name.
pub fn normalize_event_name(name: &str) -> String {
    name.trim_start_matches('@').trim_start_matches("on:").to_string()
}

/// Returns true when `name` follows `snake_case` (ASCII lowercase, digits, underscores).
pub fn is_snake_case(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }
    let mut prev_underscore = false;
    for ch in name.chars() {
        if ch == '_' {
            if prev_underscore {
                return false;
            }
            prev_underscore = true;
            continue;
        }
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() {
            return false;
        }
        prev_underscore = false;
    }
    !name.starts_with('_') && !name.ends_with('_')
}

#[cfg(test)]
mod tests {
    use super::is_snake_case;

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
}
