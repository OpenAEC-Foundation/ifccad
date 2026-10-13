use crate::*;
use ocdraw::{ifccad::*, line_pattern::SymbolSource};
use opencadcodec::{tables::TextStyle, CadDocument, Handle};
use std::collections::BTreeMap;

pub(crate) struct SourceSymbolSources {
    pub by_handle: BTreeMap<Handle, Option<IfccadSymbolSourceId>>,
}

pub(crate) fn from_cad_symbol_sources(
    source: &CadDocument,
    ids: &mut IfccadIdCounters,
    mappings: &mut IfccadMappings,
    issues: &mut Vec<IfccadDiagnostic>,
) -> Result<(Vec<IfccadSymbolSource>, SourceSymbolSources), IfccadConversionError> {
    let mut sources = Vec::new();
    let mut by_handle = BTreeMap::new();
    for row in source.text_styles.iter().filter(|s| s.is_shape_file) {
        let loc = format!("textStyle/{}", row.handle);
        if [
            row.height,
            row.width_factor,
            row.oblique_angle,
            row.last_height,
        ]
        .iter()
        .any(|v| !v.is_finite())
        {
            return Err(IfccadConversionError::InvalidStructure(
                "nonfinite shape STYLE scalar".into(),
            ));
        }
        if !row.name.is_empty() || row.last_height != 2.5 {
            issues.push(crate::diagnostics::diagnostic(
                "symbol-source-metadata",
                &loc,
                "shape STYLE name or last-used height metadata omitted",
            ));
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
                issues.push(crate::diagnostics::modification(
                    "symbol-source-locator",
                    &loc,
                    "SHX source locator discarded; basename retained",
                ));
            }
            if qualified {
                let id = ids.allocate_symbol_source_id()?;
                mappings.symbol_sources.insert(id.0, row.handle);
                sources.push(IfccadSymbolSource {
                    id,
                    source: SymbolSource::Shx {
                        file_name: file.file_name,
                    },
                });
                Some(id)
            } else {
                None
            }
        } else {
            None
        };
        if id.is_none() {
            issues.push(crate::diagnostics::diagnostic(
                "symbol-source-unsupported",
                &loc,
                "unsupported active shape STYLE properties or source filename",
            ));
        }
        by_handle.insert(row.handle, id);
    }
    Ok((sources, SourceSymbolSources { by_handle }))
}

pub(crate) fn to_cad_symbol_sources(
    source: &IfccadDocument,
    target: &mut CadDocument,
    mappings: &mut IfccadMappings,
) -> BTreeMap<IfccadSymbolSourceId, Handle> {
    let mut result = BTreeMap::new();
    for source in &source.symbol_sources {
        let SymbolSource::Shx { file_name } = &source.source;
        let mut style = TextStyle::new("");
        style.handle = target.allocate_handle();
        style.is_shape_file = true;
        style.font_file = file_name.clone();
        result.insert(source.id, style.handle);
        mappings.symbol_sources.insert(source.id.0, style.handle);
        target.text_styles.add_allow_duplicate(style);
    }
    result
}
