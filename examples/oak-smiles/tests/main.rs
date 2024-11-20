//! SMILES parsing tests.

use oak_smiles::{ast::SmilesBondOrder, parse_smiles};

#[test]
fn ethanol_is_one_fragment_with_two_bonds() {
    let root = parse_smiles("CCO").expect("ethanol parses");
    assert_eq!(root.items.len(), 1);
    assert!(root.arrow.is_none());
    let item = &root.items[0];
    assert_eq!(item.atoms.len(), 3);
    assert_eq!(item.bonds.len(), 2);
    assert_eq!(item.bonds[0].order, SmilesBondOrder::Single);
    assert_eq!(item.bonds[0].from, 0);
    assert_eq!(item.bonds[0].to, 1);
}

#[test]
fn benzene_ring_closure_resolves_into_six_aromatic_bonds() {
    let root = parse_smiles("c1ccccc1").expect("benzene parses");
    let item = &root.items[0];
    assert_eq!(item.atoms.len(), 6);
    assert_eq!(item.bonds.len(), 6, "the ring label closes the sixth bond");
    assert!(item.atoms.iter().all(|atom| atom.aromatic));
    assert!(item.bonds.iter().all(|bond| bond.order == SmilesBondOrder::Aromatic));
    assert_eq!(item.bonds[5].from, 0, "the closing bond links the first and last atoms");
    assert_eq!(item.bonds[5].to, 5);
}

#[test]
fn percent_labels_follow_the_same_table() {
    let root = parse_smiles("C%12CCCCC%12").expect("percent ring label parses");
    let item = &root.items[0];
    assert_eq!(item.atoms.len(), 6);
    assert_eq!(item.bonds.len(), 6);
}

#[test]
fn branches_return_to_the_branching_atom() {
    let root = parse_smiles("CC(C)C").expect("isobutane parses");
    let item = &root.items[0];
    assert_eq!(item.atoms.len(), 4);
    assert_eq!(item.bonds.len(), 3);
    // Allocation: 0-1, 1-2 (branch), 1-3 (after the branch closes).
    assert_eq!((item.bonds[1].from, item.bonds[1].to), (1, 2));
    assert_eq!((item.bonds[2].from, item.bonds[2].to), (1, 3));
}

#[test]
fn bracket_atoms_carry_isotope_charge_and_hydrogens() {
    let root = parse_smiles("[13CH3-]").expect("bracket atom parses");
    let atom = &root.items[0].atoms[0];
    assert_eq!(atom.element, "C");
    assert_eq!(atom.isotope, Some(13));
    assert_eq!(atom.explicit_hydrogens, Some(3));
    assert_eq!(atom.charge, -1);

    let root = parse_smiles("[Na+]").expect("sodium parses");
    let atom = &root.items[0].atoms[0];
    assert_eq!(atom.element, "Na");
    assert_eq!(atom.charge, 1);
}

#[test]
fn aromatic_nitrogen_with_hydrogen_keeps_both() {
    let root = parse_smiles("[nH]1cccc1").expect("pyrrole parses");
    let atom = &root.items[0].atoms[0];
    assert_eq!(atom.element, "n");
    assert!(atom.aromatic);
    assert_eq!(atom.explicit_hydrogens, Some(1));
}

#[test]
fn chirality_is_recorded_on_the_atom() {
    use oak_smiles::ast::SmilesChirality;

    let root = parse_smiles("N[C@@H](C)C(=O)O").expect("alanine parses");
    let atom = &root.items[0].atoms[1];
    assert_eq!(atom.chirality, Some(SmilesChirality::CounterClockwise));
}

#[test]
fn double_bond_stereo_is_recorded_on_the_bond() {
    use oak_smiles::ast::SmilesStereo;

    let root = parse_smiles("C/C=C/C").expect("trans butene parses");
    let item = &root.items[0];
    assert_eq!(item.bonds[0].stereo, Some(SmilesStereo::Up));
    assert_eq!(item.bonds[1].order, SmilesBondOrder::Double);
    assert_eq!(item.bonds[1].stereo, None, "the `=` symbol carries no direction");
    assert_eq!(item.bonds[2].stereo, Some(SmilesStereo::Up));
}

#[test]
fn dot_splits_fragments() {
    let root = parse_smiles("CC.O").expect("salt parses");
    assert_eq!(root.items.len(), 2);
    assert_eq!(root.items[0].atoms.len(), 2);
    assert_eq!(root.items[1].atoms.len(), 1);
}

#[test]
fn reaction_smiles_keeps_reactants_products_and_reagents() {
    let root = parse_smiles("CC.O>>CC=O").expect("reaction parses");
    assert_eq!(root.items.len(), 2, "reactants are split on the dot");
    assert_eq!(root.products.len(), 1);
    use oak_smiles::ast::SmilesArrowKind;
    assert_eq!(root.arrow.as_ref().map(|arrow| arrow.kind), Some(SmilesArrowKind::Double));

    let root = parse_smiles("CC>O>CC").expect("three-part reaction parses");
    assert_eq!(root.items.len(), 1);
    assert_eq!(root.reagents.len(), 1, "reagents keep their own graph");
    assert_eq!(root.reagents[0].atoms.len(), 1);
    assert_eq!(root.products.len(), 1);
    assert_eq!(root.arrow.as_ref().and_then(|arrow| arrow.above.as_deref()), Some("O"));
}

#[test]
fn unclosed_ring_label_is_refused() {
    let error = parse_smiles("C1CC").expect_err("an open ring is refused");
    assert!(error.contains("never closed"), "{error}");
}

#[test]
fn unmatched_branch_is_refused() {
    let error = parse_smiles("CC)").expect_err("a stray closing branch is refused");
    assert!(error.contains("without a matching opening"), "{error}");
}

#[test]
fn percent_parenthesised_ring_label_is_refused_by_name() {
    let error = parse_smiles("C%(12)CC%(12)").expect_err("%() is refused");
    assert!(error.contains("%(") || error.contains("ring label"), "{error}");
}

#[test]
fn wildcard_and_any_bond_are_modelled() {
    let root = parse_smiles("*~*").expect("wildcards parse");
    let item = &root.items[0];
    assert_eq!(item.atoms.len(), 2);
    assert_eq!(item.atoms[0].element, "*");
    assert_eq!(item.bonds[0].order, SmilesBondOrder::Any);
}

#[test]
fn atom_spans_select_the_authored_text() {
    let source = "CCO";
    let root = parse_smiles(source).expect("ethanol parses");
    for atom in &root.items[0].atoms {
        let text = &source[atom.span.start..atom.span.end];
        assert_eq!(text.len(), 1);
        assert!(text.chars().all(|character| character.is_ascii_alphabetic()));
    }
}
