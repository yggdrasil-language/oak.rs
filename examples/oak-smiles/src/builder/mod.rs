//! Builder turning a SMILES green tree into the typed AST.

use std::collections::BTreeMap;

use core::range::Range;

use oak_core::{
    Builder, BuilderCache, GreenNode, GreenTree, OakDiagnostics, OakError, Parser, SourceText, TextEdit,
    parser::session::ParseSession,
    source::Source,
};

use crate::{
    ast::{
        SmilesArrow, SmilesArrowKind, SmilesAtom, SmilesBond, SmilesBondOrder, SmilesChirality, SmilesItem, SmilesRoot,
        SmilesStereo,
    },
    language::SmilesLanguage,
    parser::{SmilesParser, element_type::SmilesElementType},
};

/// Builder for the SMILES typed AST.
#[derive(Default)]
pub struct SmilesBuilder;

impl Builder<SmilesLanguage> for SmilesBuilder {
    fn build<'a, S: Source + ?Sized>(
        &self,
        source: &S,
        edits: &[TextEdit],
        _cache: &'a mut impl BuilderCache<SmilesLanguage>,
    ) -> OakDiagnostics<SmilesRoot> {
        let parser = SmilesParser;
        let mut cache = ParseSession::<SmilesLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut cache);

        match parse_result.result {
            Ok(green_tree) => {
                let text = source.get_text_in((0..source.length()).into()).into_owned();
                let source_text = SourceText::new(text);
                let mut diagnostics = parse_result.diagnostics;
                match self.build_root(green_tree, &source_text) {
                    Ok(root) => OakDiagnostics { result: Ok(root), diagnostics },
                    Err(error) => {
                        diagnostics.push(error.clone());
                        OakDiagnostics { result: Err(error), diagnostics }
                    }
                }
            }
            Err(error) => OakDiagnostics { result: Err(error), diagnostics: parse_result.diagnostics },
        }
    }
}

/// One fragment under construction.
#[derive(Default)]
struct Item {
    atoms: Vec<SmilesAtom>,
    bonds: Vec<SmilesBond>,
    start: usize,
    end: usize,
    touched: bool,
}

impl Item {
    fn into_ast(self) -> SmilesItem {
        SmilesItem {
            atoms: self.atoms,
            bonds: self.bonds,
            span: (self.start..self.end.max(self.start)).into(),
        }
    }
}

/// Traversal state shared by both sides of the arrow.
struct Traversal {
    current: Option<usize>,
    branches: Vec<usize>,
    pending: Option<(SmilesBondOrder, Option<SmilesStereo>)>,
    rings: BTreeMap<String, (usize, Option<(SmilesBondOrder, Option<SmilesStereo>)>)>,
}

impl Default for Traversal {
    fn default() -> Self {
        Self { current: None, branches: Vec::new(), pending: None, rings: BTreeMap::new() }
    }
}

impl Traversal {
    fn reset(&mut self) {
        *self = Self::default();
    }
}

impl SmilesBuilder {
    fn build_root(&self, green_tree: &GreenNode<SmilesLanguage>, source: &SourceText) -> Result<SmilesRoot, OakError> {
        if green_tree.kind != SmilesElementType::SourceFile {
            return Err(OakError::internal_error("the SMILES syntax tree is missing its root element"));
        }

        let document_start = 0usize;
        let mut document_end = 0usize;
        let mut reactants = Vec::new();
        let mut reagents = Vec::new();
        let mut products = Vec::new();
        let mut item = Item::default();
        let mut traversal = Traversal::default();
        let mut phase = Phase::Reactants;
        let mut arrow: Option<SmilesArrow> = None;
        let mut reagent_start: Option<usize> = None;
        let mut reagent_end = 0usize;
        let mut offset = 0usize;

        for child in green_tree.children {
            let GreenTree::Node(node) = child
            else {
                if let GreenTree::Leaf(leaf) = child {
                    offset += leaf.length as usize;
                }
                continue;
            };
            let start = offset;
            let end = offset + node.byte_length as usize;
            offset = end;
            document_end = end;

            match node.kind {
                SmilesElementType::OrganicAtom | SmilesElementType::BracketAtom => {
                    let text = source.get_text_in((start..end).into()).into_owned();
                    let atom = parse_atom(&text, (start..end).into(), source)?;
                    if !item.touched {
                        item.touched = true;
                        item.start = start;
                    }
                    item.end = end;
                    let index = item.atoms.len();
                    item.atoms.push(atom);
                    if let Some(previous) = traversal.current {
                        let order = traversal
                            .pending
                            .take()
                            .unwrap_or_else(|| implied_order(&item.atoms[previous], &item.atoms[index]));
                        item.bonds.push(SmilesBond {
                            from: previous,
                            to: index,
                            order: order.0,
                            stereo: order.1,
                            span: (start..end).into(),
                        });
                    }
                    traversal.current = Some(index);
                }
                SmilesElementType::Bond => {
                    let text = source.get_text_in((start..end).into()).into_owned();
                    traversal.pending = Some(parse_bond(&text, (start..end).into(), source)?);
                }
                SmilesElementType::BranchOpen => {
                    let Some(current) = traversal.current
                    else {
                        return Err(OakError::syntax_error(
                            "a branch cannot open before the first atom",
                            start,
                            source.source_id(),
                        ));
                    };
                    traversal.branches.push(current);
                }
                SmilesElementType::BranchClose => {
                    let Some(previous) = traversal.branches.pop()
                    else {
                        return Err(OakError::syntax_error(
                            "a branch closes without a matching opening",
                            start,
                            source.source_id(),
                        ));
                    };
                    traversal.pending = None;
                    traversal.current = Some(previous);
                }
                SmilesElementType::RingLabel => {
                    let text = source.get_text_in((start..end).into()).into_owned();
                    let Some(label) = ring_label(&text) else {
                        return Err(OakError::syntax_error(
                            format!("ring label `{text}` is outside the modelled subset, only `0`-`9` and `%nn` are accepted"),
                            start,
                            source.source_id(),
                        ));
                    };
                    let Some(current) = traversal.current
                    else {
                        return Err(OakError::syntax_error(
                            "a ring label needs an atom before it",
                            start,
                            source.source_id(),
                        ));
                    };
                    match traversal.rings.remove(&label) {
                        Some((other, order)) => {
                            let order = traversal.pending.take().or(order).unwrap_or_else(|| {
                                implied_order(&item.atoms[other], &item.atoms[current])
                            });
                            item.bonds.push(SmilesBond {
                                from: other,
                                to: current,
                                order: order.0,
                                stereo: order.1,
                                span: (start..end).into(),
                            });
                        }
                        None => {
                            traversal.rings.insert(label, (current, traversal.pending.take()));
                        }
                    }
                }
                SmilesElementType::Dot => {
                    self.finish_item(&mut item, &mut traversal, phase, start, source, &mut reactants, &mut reagents, &mut products)?;
                }
                SmilesElementType::Arrow => {
                    let text = source.get_text_in((start..end).into()).into_owned();
                    let double = text == ">>";
                    if let Some(reagent_start) = reagent_start.take() {
                        // The second boundary closes the reagent text.
                        reagent_end = start;
                        arrow = Some(SmilesArrow {
                            kind: SmilesArrowKind::Single,
                            above: Some(
                                source
                                    .get_text_in((reagent_start..reagent_end).into())
                                    .trim()
                                    .to_string(),
                            )
                            .filter(|text| !text.is_empty()),
                            below: None,
                            span: (reagent_start..end).into(),
                        });
                    }
                    self.finish_item(&mut item, &mut traversal, phase, start, source, &mut reactants, &mut reagents, &mut products)?;
                    if double {
                        arrow = Some(SmilesArrow {
                            kind: SmilesArrowKind::Double,
                            above: None,
                            below: None,
                            span: (start..end).into(),
                        });
                        phase = Phase::Products;
                    }
                    else if phase == Phase::Reactants {
                        phase = Phase::Reagents;
                        reagent_start = Some(end);
                    }
                    else {
                        phase = Phase::Products;
                    }
                }
                _ => {}
            }
        }

        self.finish_item(&mut item, &mut traversal, phase, document_end, source, &mut reactants, &mut reagents, &mut products)?;
        if let Some(reagent_start) = reagent_start {
            arrow = Some(SmilesArrow {
                kind: SmilesArrowKind::Single,
                above: Some(source.get_text_in((reagent_start..document_end).into()).trim().to_string())
                    .filter(|text| !text.is_empty()),
                below: None,
                span: (reagent_start..document_end).into(),
            });
        }

        Ok(SmilesRoot {
            items: reactants,
            arrow,
            reagents,
            products,
            span: (document_start..document_end).into(),
        })
    }

    /// Close the fragment under construction and reset the traversal.
    #[allow(clippy::too_many_arguments)]
    fn finish_item(
        &self,
        item: &mut Item,
        traversal: &mut Traversal,
        phase: Phase,
        offset: usize,
        source: &SourceText,
        reactants: &mut Vec<SmilesItem>,
        reagents: &mut Vec<SmilesItem>,
        products: &mut Vec<SmilesItem>,
    ) -> Result<(), OakError> {
        if !item.touched {
            return Ok(());
        }
        if let Some((label, _)) = traversal.rings.iter().next() {
            return Err(OakError::syntax_error(
                format!("ring label `{label}` is never closed in its fragment"),
                offset,
                source.source_id(),
            ));
        }
        if !traversal.branches.is_empty() {
            return Err(OakError::syntax_error("a branch is never closed", offset, source.source_id()));
        }
        let finished = std::mem::take(item).into_ast();
        match phase {
            Phase::Products => products.push(finished),
            Phase::Reagents => reagents.push(finished),
            Phase::Reactants => reactants.push(finished),
        }
        traversal.reset();
        Ok(())
    }
}

/// Which side of the arrow the current fragment belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Reactants,
    Reagents,
    Products,
}

/// Parse one atom token.
fn parse_atom(text: &str, span: Range<usize>, source: &SourceText) -> Result<SmilesAtom, OakError> {
    if text.starts_with('[') {
        parse_bracket_atom(text, span, source)
    }
    else {
        let first = text.chars().next().unwrap_or('*');
        if !is_organic_symbol(text) {
            return Err(OakError::syntax_error(
                format!("`{text}` is not an organic-subset atom or a bracket atom"),
                span.start,
                source.source_id(),
            ));
        }
        Ok(SmilesAtom {
            element: text.to_string(),
            aromatic: first.is_ascii_lowercase(),
            charge: 0,
            isotope: None,
            explicit_hydrogens: None,
            chirality: None,
            span,
        })
    }
}

/// Whether the text is an accepted unbracketed atom symbol.
fn is_organic_symbol(text: &str) -> bool {
    matches!(
        text,
        "B" | "C" | "N" | "O" | "P" | "S" | "F" | "I" | "Cl" | "Br" | "b" | "c" | "n" | "o" | "p" | "s" | "H" | "*"
    )
}

fn parse_bracket_atom(text: &str, span: Range<usize>, source: &SourceText) -> Result<SmilesAtom, OakError> {
    let inner = text.trim_start_matches('[').trim_end_matches(']');
    if inner.is_empty() {
        return Err(OakError::syntax_error("an empty bracket atom has no element", span.start, source.source_id()));
    }

    let bytes = inner.as_bytes();
    let mut index = 0usize;

    let isotope_start = index;
    while index < bytes.len() && bytes[index].is_ascii_digit() {
        index += 1;
    }
    let isotope = if index > isotope_start {
        inner[isotope_start..index].parse::<u16>().ok()
    }
    else {
        None
    };

    let element = if index < bytes.len() && bytes[index] == b'*' {
        index += 1;
        "*".to_string()
    }
    else if index < bytes.len() && bytes[index].is_ascii_alphabetic() {
        let start = index;
        index += 1;
        // A second lowercase letter belongs to the symbol (`Na`, `Cl`), except
        // `h`, which is the explicit-hydrogen marker (`nH`, `NH`).
        if index < bytes.len() && bytes[index].is_ascii_lowercase() && bytes[index] != b'h' {
            index += 1;
        }
        inner[start..index].to_string()
    }
    else {
        return Err(OakError::syntax_error(
            "a bracket atom is missing its element symbol",
            span.start,
            source.source_id(),
        ));
    };
    let aromatic = element.chars().next().is_some_and(|character| character.is_ascii_lowercase());

    let mut charge = 0i8;
    let mut explicit_hydrogens = None;
    let mut chirality = None;

    while index < bytes.len() {
        match bytes[index] {
            b'@' => {
                let start = index;
                while index < bytes.len() && bytes[index] == b'@' {
                    index += 1;
                }
                chirality = Some(if index - start >= 2 {
                    SmilesChirality::CounterClockwise
                }
                else {
                    SmilesChirality::Clockwise
                });
            }
            b'H' | b'h' => {
                index += 1;
                let start = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                let count = if start == index { 1 } else { inner[start..index].parse::<u8>().unwrap_or(1) };
                explicit_hydrogens = Some(count);
            }
            sign @ (b'+' | b'-') => {
                let direction = if sign == b'+' { 1i8 } else { -1i8 };
                let start = index;
                while index < bytes.len() && bytes[index] == sign {
                    index += 1;
                }
                let repeated = (index - start) as i8;
                let digits_start = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                let magnitude = if digits_start == index {
                    repeated
                }
                else {
                    inner[digits_start..index].parse::<i8>().unwrap_or(repeated)
                };
                charge = direction * magnitude.max(1);
            }
            _ => {
                return Err(OakError::syntax_error(
                    format!("bracket atom part `{}` is outside the modelled subset", &inner[index..index + 1]),
                    span.start,
                    source.source_id(),
                ));
            }
        }
    }

    Ok(SmilesAtom { element, aromatic, charge, isotope, explicit_hydrogens, chirality, span })
}

/// Parse a bond symbol.
fn parse_bond(text: &str, span: Range<usize>, source: &SourceText) -> Result<(SmilesBondOrder, Option<SmilesStereo>), OakError> {
    match text {
        "-" => Ok((SmilesBondOrder::Single, None)),
        "=" => Ok((SmilesBondOrder::Double, None)),
        "#" => Ok((SmilesBondOrder::Triple, None)),
        ":" => Ok((SmilesBondOrder::Aromatic, None)),
        "~" => Ok((SmilesBondOrder::Any, None)),
        "/" => Ok((SmilesBondOrder::Single, Some(SmilesStereo::Up))),
        "\\" => Ok((SmilesBondOrder::Single, Some(SmilesStereo::Down))),
        other => Err(OakError::syntax_error(
            format!("bond symbol `{other}` is outside the modelled subset"),
            span.start,
            source.source_id(),
        )),
    }
}

/// The order implied when no symbol was written.
fn implied_order(from: &SmilesAtom, to: &SmilesAtom) -> (SmilesBondOrder, Option<SmilesStereo>) {
    if from.aromatic && to.aromatic {
        (SmilesBondOrder::Aromatic, None)
    }
    else {
        (SmilesBondOrder::Single, None)
    }
}

/// Normalise a ring label token into its digit string.
fn ring_label(text: &str) -> Option<String> {
    let digits = text.strip_prefix('%').unwrap_or(text);
    if digits.is_empty() || !digits.chars().all(|character| character.is_ascii_digit()) {
        return None;
    }
    Some(digits.to_string())
}
