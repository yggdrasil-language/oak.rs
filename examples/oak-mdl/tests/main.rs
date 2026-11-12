//! MDL connection-table parsing tests.

use oak_mdl::{ast::MdlBondOrder, parse_mdl};

/// A V2000 molfile for water, with an exact 3-line header and a counts line.
fn water() -> String {
    let mut text = String::new();
    text.push_str("water\n");
    text.push_str("  oak     2D\n");
    text.push_str("\n");
    text.push_str("  1  0  0  0  0  0  0  0  0  0999 V2000\n");
    text.push_str("    0.0000    0.0000    0.0000 O   0  0  0  0  0  0  0  0  0  0  0  0\n");
    text
}

#[test]
fn parses_one_record_with_its_header_and_atoms() {
    let root = parse_mdl(&water()).expect("water parses");
    assert_eq!(root.records.len(), 1);
    let record = &root.records[0];
    assert_eq!(record.name.as_deref(), Some("water"));
    assert_eq!(record.atom_count, 1);
    assert_eq!(record.bond_count, 0);
    assert_eq!(record.atoms.len(), 1);
    assert_eq!(record.atoms[0].element, "O");
    assert_eq!(record.atoms[0].charge, 0);
}

#[test]
fn benzene_keeps_every_bond_order_and_span() {
    let mut text = String::new();
    text.push_str("benzene\n  oak\n\n");
    text.push_str("  6  6  0  0  0  0  0  0  0  0999 V2000\n");
    for index in 0..6 {
        let angle = std::f64::consts::TAU * index as f64 / 6.0;
        text.push_str(&format!(
            "{:10.4}{:10.4}{:10.4} C   0  0  0  0  0  0  0  0  0  0  0  0\n",
            angle.cos(),
            angle.sin(),
            0.0
        ));
    }
    for index in 0..6 {
        let from = index + 1;
        let to = (index + 1) % 6 + 1;
        text.push_str(&format!("{:3}{:3}{:3}{:3}  0  0  0  0  0  0  0\n", from, to, 4, 0));
    }

    let root = parse_mdl(&text).expect("benzene parses");
    let record = &root.records[0];
    assert_eq!(record.atoms.len(), 6);
    assert_eq!(record.bonds.len(), 6);
    assert!(record.bonds.iter().all(|bond| bond.order == MdlBondOrder::Aromatic));
    assert_eq!(record.bonds[0].from, 1, "bond endpoints stay 1-based");
    assert_eq!(record.bonds[0].to, 2);
    let bond_span = &record.bonds[0].span;
    let text_of_bond = &text[bond_span.start..bond_span.end];
    assert!(text_of_bond.starts_with("  1  2  4"), "the span selects the authored line: {text_of_bond:?}");
}

#[test]
fn sdf_records_split_on_the_record_separator() {
    let mut text = water();
    text.push_str("$$$$\n");
    text.push_str(&water());

    let root = parse_mdl(&text).expect("two records parse");
    assert_eq!(root.records.len(), 2);
    assert!(root.records.iter().all(|record| record.atoms.len() == 1));
}

#[test]
fn charge_property_is_parsed_alongside_the_raw_line() {
    let mut text = water();
    text.push_str("M  CHG  1   1  -1\n");

    let root = parse_mdl(&text).expect("charge property parses");
    let record = &root.records[0];
    assert_eq!(record.properties.len(), 1);
    let property = &record.properties[0];
    assert_eq!(property.kind, "CHG");
    assert_eq!(property.count, 1);
    assert_eq!(property.entries.len(), 1);
    assert_eq!(property.entries[0].atom, 1);
    assert_eq!(property.entries[0].value, -1);
    assert!(property.raw.starts_with("M  CHG"));
}

#[test]
fn data_fields_collect_their_value_lines() {
    let mut text = water();
    text.push_str("> <PUBCHEM_ID>\n");
    text.push_str("962\n");
    text.push_str("$$$$\n");

    let root = parse_mdl(&text).expect("data field parses");
    let record = &root.records[0];
    assert_eq!(record.data.len(), 1);
    assert_eq!(record.data[0].name, "PUBCHEM_ID");
    assert_eq!(record.data[0].value, "962");
}

#[test]
fn v3000_is_refused_with_a_positioned_error() {
    let mut text = String::new();
    text.push_str("v3000\n  oak\n\n");
    text.push_str("  0  0  0     0  0            999 V3000\n");
    let error = parse_mdl(&text).expect_err("V3000 is refused");
    assert!(error.contains("V3000"), "{error}");
}

#[test]
fn reaction_files_are_refused_instead_of_flattened() {
    let error = parse_mdl("$RXN\n\n\n\n").expect_err("$RXN is refused");
    assert!(error.contains("$RXN"), "{error}");
}

#[test]
fn bond_endpoint_outside_the_atom_count_is_refused() {
    let mut text = String::new();
    text.push_str("bad\n  oak\n\n");
    text.push_str("  1  1  0  0  0  0  0  0  0  0999 V2000\n");
    text.push_str("    0.0000    0.0000    0.0000 C   0  0  0  0  0  0  0  0  0  0  0  0\n");
    text.push_str("  1  2  1  0  0  0  0\n");

    let error = parse_mdl(&text).expect_err("a dangling bond is refused");
    assert!(error.contains("outside the declared 1 atoms"), "{error}");
}

#[test]
fn empty_source_produces_no_records() {
    let root = parse_mdl("").expect("an empty source is not an error");
    assert!(root.records.is_empty());
}
