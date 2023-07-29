use oak_ini::{IniLanguage, parse, parse_with};

#[test]
fn typed_default_parses_simple_assignment() {
    let root = parse("[A]\nB=1\n").expect("parse");
    assert_eq!(root.sections.len(), 1);
    assert_eq!(root.sections[0].name, "A");
    assert_eq!(root.sections[0].properties.len(), 1);
    assert_eq!(root.sections[0].properties[0].key, "B");
    assert_eq!(root.sections[0].properties[0].value, "1");
}

#[test]
fn westwood_allows_numeric_keys_and_list_values() {
    let src = "\
[VehicleTypes]
0=MTNK
1=HTNK

[MTNK]
Strength=400
Owner=Britishs,Americans
Armor=heavy
";
    let root = parse_with(src, &IniLanguage::westwood()).expect("westwood parse");
    let vehicles = root.sections.iter().find(|s| s.name == "VehicleTypes").unwrap();
    assert_eq!(vehicles.properties[0].key, "0");
    assert_eq!(vehicles.properties[0].value, "MTNK");
    let mtnk = root.sections.iter().find(|s| s.name == "MTNK").unwrap();
    assert_eq!(mtnk.properties.iter().find(|p| p.key == "Owner").unwrap().value, "Britishs,Americans");
}

#[test]
fn westwood_preserves_duplicate_keys() {
    let root = parse_with("[D]\nK=first\nK=second\n", &IniLanguage::westwood()).unwrap();
    let values: Vec<_> = root.sections[0].properties.iter().map(|p| p.value.as_str()).collect();
    assert_eq!(values, ["first", "second"]);
}
