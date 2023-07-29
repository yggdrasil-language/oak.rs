# Ini Language

Dialect configuration for INI parsing. INI has no single standard, so behavior is selected via `IniLanguage` flags:

- `numeric_keys` — allow `0=Type` list keys
- `hash_comments` — treat `#` as a line comment (in addition to `;`)
- `value_style` — `Typed` (token literals) vs `LineRemainder` (classic / Westwood rest-of-line values)

Presets:

- `IniLanguage::default()` — typed values, `#` comments
- `IniLanguage::westwood()` — Red Alert style rules/maps/art INI
