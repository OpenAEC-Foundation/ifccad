use crate::*;
use ocdraw::{line_pattern::SymbolSource, ocdraw::*};
use opencadcodec::{tables::TextStyle, CadDocument, Handle};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) struct ExportSymbolSourceMap {
    pub by_handle: BTreeMap<Handle, Option<SymbolSourceId>>,
}

pub(crate) fn export_symbol_sources(
    source: &CadDocument,
    builder: &mut OcdrawBuilder,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<ExportSymbolSourceMap, CadToOcdrawError> {
    let mut by_handle = BTreeMap::new();
    let mut seen = BTreeSet::new();
    for row in source.text_styles.iter() {
        if !row.handle.is_null() && !seen.insert(row.handle) {
            return Err(super::line_pattern::invalid("duplicate STYLE handle"));
        }
        if !row.is_shape_file {
            continue;
        }
        let location = CadToOcdrawDiagnosticSource::Table {
            kind: format!("text_styles/shape/{}", row.handle),
        };
        if !row.name.is_empty() || row.last_height != 2.5 {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                location.clone(),
                CadToOcdrawAction::PartiallyExported,
                vec![CadToOcdrawLossReason::UnsupportedSemantic {
                    name: "shape STYLE name or last-used height metadata".into(),
                }],
            ));
        }
        if [
            row.height,
            row.width_factor,
            row.oblique_angle,
            row.last_height,
        ]
        .iter()
        .any(|v| !v.is_finite())
        {
            return Err(super::line_pattern::invalid("nonfinite shape STYLE scalar"));
        }
        let qualified = row.height == 0.
            && row.width_factor == 1.
            && row.oblique_angle == 0.
            && !row.flags.backward
            && !row.flags.upside_down
            && !row.is_vertical
            && !row.annotative
            && row.big_font_file.is_empty()
            && row.true_type_font.is_empty()
            && !row.xref_dependent
            && row.xref_block_record_handle.is_null();
        let file = cad_presentation_convert::shx_file_name_from_cad(&row.font_file);
        let id = if let Ok(file) = file {
            if file.discarded_locator {
                diagnostics.push(CadToOcdrawDiagnostic::loss(
                    location.clone(),
                    CadToOcdrawAction::PartiallyExported,
                    vec![CadToOcdrawLossReason::UnsupportedSemantic {
                        name: "SHX source locator discarded; basename retained".into(),
                    }],
                ));
            }
            if qualified {
                Some(builder.add_symbol_source(SymbolSource::Shx {
                    file_name: file.file_name,
                })?)
            } else {
                None
            }
        } else {
            None
        };
        if id.is_none() {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                location,
                CadToOcdrawAction::Skipped,
                vec![CadToOcdrawLossReason::UnsupportedSemantic {
                    name: "unsupported shape STYLE properties or source filename".into(),
                }],
            ));
        }
        if !row.handle.is_null() {
            by_handle.insert(row.handle, id);
        }
    }
    Ok(ExportSymbolSourceMap { by_handle })
}

pub(crate) fn import_symbol_sources(
    source: &OcdrawDocument,
    target: &mut CadDocument,
) -> BTreeMap<SymbolSourceId, Handle> {
    let mut map = BTreeMap::new();
    for source in &source.symbol_sources {
        let SymbolSource::Shx { file_name } = &source.source;
        let mut row = TextStyle::new("");
        row.handle = target.allocate_handle();
        row.is_shape_file = true;
        row.font_file = file_name.clone();
        map.insert(source.id, row.handle);
        target.text_styles.add_allow_duplicate(row);
    }
    map
}
