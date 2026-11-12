//! Builder turning an MDL green tree into the typed AST.

use core::range::Range;

use oak_core::{
    Builder, BuilderCache, GreenNode, GreenTree, OakDiagnostics, OakError, Parser, SourceText, TextEdit,
    parser::session::ParseSession,
    source::Source,
};

use crate::{
    ast::{MdlAtom, MdlBond, MdlBondOrder, MdlDataField, MdlProperty, MdlPropertyEntry, MdlRecord, MdlRoot, MdlVersion},
    language::MdlLanguage,
    parser::{MdlParser, element_type::MdlElementType},
};

/// Builder for the MDL typed AST.
#[derive(Default)]
pub struct MdlBuilder;

impl Builder<MdlLanguage> for MdlBuilder {
    fn build<'a, S: Source + ?Sized>(
        &self,
        source: &S,
        edits: &[TextEdit],
        _cache: &'a mut impl BuilderCache<MdlLanguage>,
    ) -> OakDiagnostics<MdlRoot> {
        let parser = MdlParser;
        let mut cache = ParseSession::<MdlLanguage>::default();
        let parse_result = parser.parse(source, edits, &mut cache);

        match parse_result.result {
            Ok(green_tree) => {
                let text = source.get_text_in((0..source.length()).into()).into_owned();
                let source_text = SourceText::new(text);
                let mut diagnostics = parse_result.diagnostics;
                match self.build_root(green_tree, &source_text, &mut diagnostics) {
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

/// A record under construction.
#[derive(Default)]
struct Pending {
    name: Option<String>,
    program: Option<String>,
    comment: Option<String>,
    header_lines: usize,
    counts: Option<(usize, usize)>,
    atoms: Vec<MdlAtom>,
    bonds: Vec<MdlBond>,
    properties: Vec<MdlProperty>,
    data: Vec<MdlDataField>,
    active_data: Option<(String, Vec<String>, Range<usize>)>,
    start: usize,
    end: usize,
}

impl MdlBuilder {
    fn build_root(
        &self,
        green_tree: &GreenNode<MdlLanguage>,
        source: &SourceText,
        diagnostics: &mut Vec<OakError>,
    ) -> Result<MdlRoot, OakError> {
        if green_tree.kind != MdlElementType::SourceFile {
            return Err(OakError::internal_error("the MDL syntax tree is missing its root element"));
        }

        let mut records = Vec::new();
        let mut pending = Pending::default();
        let mut offset = 0usize;

        for child in green_tree.children {
            match child {
                GreenTree::Node(node) => {
                    let start = offset;
                    let end = offset + node.byte_length as usize;
                    offset = end;
                    self.push_element(
                        node.kind,
                        (start..end).into(),
                        source,
                        &mut pending,
                        &mut records,
                        diagnostics,
                    )?;
                }
                GreenTree::Leaf(leaf) => {
                    offset += leaf.length as usize;
                }
            }
        }
        self.flush(&mut pending, &mut records);

        Ok(MdlRoot { records })
    }

    #[allow(clippy::too_many_arguments)]
    fn push_element(
        &self,
        kind: MdlElementType,
        span: Range<usize>,
        source: &SourceText,
        pending: &mut Pending,
        records: &mut Vec<MdlRecord>,
        diagnostics: &mut Vec<OakError>,
    ) -> Result<(), OakError> {
        if pending.start == 0 && pending.header_lines == 0 && pending.counts.is_none() && pending.atoms.is_empty() {
            pending.start = span.start;
        }
        pending.end = span.end;

        match kind {
            MdlElementType::RecordSeparator => {
                self.flush(pending, records);
                Ok(())
            }
            MdlElementType::ReactionMarker => Err(OakError::syntax_error(
                "`$RXN` and `$RDFILE` reaction files are not modelled yet, so the source is refused instead of flattened into molecules",
                span.start,
                source.source_id(),
            )),
            MdlElementType::DataFieldName => {
                self.flush_data(pending);
                let line = line_text(source, span);
                let name = field_name(&line).ok_or_else(|| {
                    OakError::syntax_error("a data header needs a `<NAME>` label", span.start, source.source_id())
                })?;
                pending.active_data = Some((name, Vec::new(), span.clone()));
                Ok(())
            }
            MdlElementType::PropertyLine => {
                self.flush_data(pending);
                let line = line_text(source, span);
                pending.properties.push(parse_property(&line, span));
                Ok(())
            }
            MdlElementType::HeaderLine => {
                let line = line_text(source, span);
                if pending.active_data.is_some() {
                    if line.trim().is_empty() {
                        self.flush_data(pending);
                    }
                    else if let Some((_, values, end)) = &mut pending.active_data {
                        values.push(line);
                        end.end = span.end;
                    }
                    return Ok(());
                }
                if line.trim().is_empty() {
                    if pending.counts.is_none() && pending.header_lines < 3 {
                        pending.header_lines += 1;
                    }
                    return Ok(());
                }
                match pending.counts {
                    None if pending.header_lines < 3 => {
                        match pending.header_lines {
                            0 => pending.name = non_empty(&line),
                            1 => pending.program = non_empty(&line),
                            _ => pending.comment = non_empty(&line),
                        }
                        pending.header_lines += 1;
                    }
                    None => {
                        pending.counts = Some(parse_counts(&line, span, source)?);
                    }
                    Some((atom_count, bond_count)) => {
                        if pending.atoms.len() < atom_count {
                            pending.atoms.push(parse_atom(&line, span, source)?);
                        }
                        else if pending.bonds.len() < bond_count {
                            pending.bonds.push(parse_bond(&line, span, atom_count, source)?);
                        }
                        else {
                            diagnostics.push(OakError::syntax_error(
                                "a content line after the declared atoms and bonds does not start with `M`, `>`, or `$$$$`",
                                span.start,
                                source.source_id(),
                            ));
                        }
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Close the record under construction, if it declared a counts line.
    fn flush(&self, pending: &mut Pending, records: &mut Vec<MdlRecord>) {
        self.flush_data(pending);
        let Some((atom_count, bond_count)) = pending.counts
        else {
            *pending = Pending::default();
            return;
        };
        records.push(MdlRecord {
            name: pending.name.clone(),
            program: pending.program.clone(),
            comment: pending.comment.clone(),
            version: MdlVersion::V2000,
            atom_count,
            bond_count,
            atoms: std::mem::take(&mut pending.atoms),
            bonds: std::mem::take(&mut pending.bonds),
            properties: std::mem::take(&mut pending.properties),
            data: std::mem::take(&mut pending.data),
            span: (pending.start..pending.end.max(pending.start)).into(),
        });
        *pending = Pending::default();
    }

    fn flush_data(&self, pending: &mut Pending) {
        if let Some((name, values, span)) = pending.active_data.take() {
            pending.data.push(MdlDataField { name, value: values.join("\n"), span });
        }
    }
}

/// The source text of one element, without the trailing line break.
fn line_text(source: &SourceText, span: Range<usize>) -> String {
    let text = source.get_text_in(span).into_owned();
    text.trim_end_matches(['\n', '\r']).to_string()
}

/// Parse the counts line into `(atom_count, bond_count)` and refuse V3000.
fn parse_counts(line: &str, span: Range<usize>, source: &SourceText) -> Result<(usize, usize), OakError> {
    if line.contains("V3000") {
        return Err(OakError::syntax_error(
            "V3000 connection tables are not modelled yet, so the source is refused instead of read as V2000",
            span.start,
            source.source_id(),
        ));
    }
    let atom_count = columns(line, 0, 3).trim();
    let bond_count = columns(line, 3, 6).trim();
    let atom_count = atom_count.parse::<usize>().map_err(|_| {
        OakError::syntax_error(
            format!("counts line atom field `{atom_count}` is not a number"),
            span.start,
            source.source_id(),
        )
    })?;
    let bond_count = bond_count.parse::<usize>().map_err(|_| {
        OakError::syntax_error(
            format!("counts line bond field `{bond_count}` is not a number"),
            span.start,
            source.source_id(),
        )
    })?;
    Ok((atom_count, bond_count))
}

fn non_empty(text: &str) -> Option<String> {
    let trimmed = text.trim();
    (!trimmed.is_empty()).then(|| trimmed.to_string())
}

/// A fixed-column slice; missing columns read as empty.
fn columns(line: &str, start: usize, end: usize) -> &str {
    if start >= line.len() {
        return "";
    }
    let end = end.min(line.len());
    &line[start..end]
}

fn field_name(line: &str) -> Option<String> {
    let open = line.find('<')?;
    let close = line[open + 1..].find('>')? + open + 1;
    let name = line[open + 1..close].trim();
    (!name.is_empty()).then(|| name.to_string())
}

fn parse_atom(line: &str, span: Range<usize>, source: &SourceText) -> Result<MdlAtom, OakError> {
    let element = columns(line, 31, 34).trim();
    if element.is_empty() {
        return Err(OakError::syntax_error(
            "an atom line is missing the element symbol in columns 32-34",
            span.start,
            source.source_id(),
        ));
    }
    let coordinate = |start: usize, end: usize| -> Result<f64, OakError> {
        let raw = columns(line, start, end).trim();
        raw.parse::<f64>().map_err(|_| {
            OakError::syntax_error(
                format!("atom coordinate `{raw}` is not a number"),
                span.start,
                source.source_id(),
            )
        })
    };
    let integer = |start: usize, end: usize| -> i16 {
        columns(line, start, end).trim().parse::<i16>().unwrap_or(0)
    };
    Ok(MdlAtom {
        element: element.to_string(),
        x: coordinate(0, 10)?,
        y: coordinate(10, 20)?,
        z: coordinate(20, 30)?,
        mass_difference: integer(34, 36),
        charge: decode_charge(integer(36, 39)),
        stereo: integer(39, 42),
        span,
    })
}

fn parse_bond(line: &str, span: Range<usize>, atom_count: usize, source: &SourceText) -> Result<MdlBond, OakError> {
    let index = |start: usize, end: usize| -> Result<usize, OakError> {
        let raw = columns(line, start, end).trim();
        raw.parse::<usize>().map_err(|_| {
            OakError::syntax_error(
                format!("bond endpoint `{raw}` is not an atom index"),
                span.start,
                source.source_id(),
            )
        })
    };
    let from = index(0, 3)?;
    let to = index(3, 6)?;
    if from == 0 || to == 0 || from > atom_count || to > atom_count {
        return Err(OakError::syntax_error(
            format!("bond endpoint ({from}, {to}) is outside the declared {atom_count} atoms"),
            span.start,
            source.source_id(),
        ));
    }
    let order = match columns(line, 6, 9).trim().parse::<u8>().unwrap_or(0) {
        1 => MdlBondOrder::Single,
        2 => MdlBondOrder::Double,
        3 => MdlBondOrder::Triple,
        4 => MdlBondOrder::Aromatic,
        other => MdlBondOrder::Other(other),
    };
    let stereo = columns(line, 9, 12).trim().parse::<i16>().unwrap_or(0);
    Ok(MdlBond { from, to, order, stereo, span })
}

/// Decode the MDL charge field into a signed charge.
fn decode_charge(code: i16) -> i8 {
    match code {
        1 => 3,
        2 => 2,
        3 => 1,
        5 => -1,
        6 => -2,
        7 => -3,
        _ => 0,
    }
}

fn parse_property(line: &str, span: Range<usize>) -> MdlProperty {
    let kind = columns(line, 3, 6).trim().to_string();
    let count = columns(line, 6, 9).trim().parse::<usize>().unwrap_or(0);
    let mut numbers = line
        .get(9..)
        .unwrap_or_default()
        .split_whitespace()
        .filter_map(|value| value.parse::<i16>().ok())
        .collect::<Vec<_>>();
    let mut entries = Vec::new();
    while numbers.len() >= 2 {
        let value = numbers.remove(1);
        let atom = numbers.remove(0);
        if atom > 0 {
            entries.push(MdlPropertyEntry { atom: atom as usize, value });
        }
    }
    MdlProperty { kind, count, entries, raw: line.trim().to_string(), span }
}
