use crate::source::{
    CadToOcdrawAction, CadToOcdrawDiagnostic, CadToOcdrawDiagnosticSource, CadToOcdrawLossReason,
};
use cad_presentation_convert::*;
use ocdraw::ocdraw::*;
use opencadcodec::objects::{KnownXRecordKind as K, XRecordValue};
use opencadcodec::{CadDocument, Handle};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn from_cad(
    source: &CadDocument,
    drawing: &mut OcdrawDocument,
    entities: &BTreeMap<Handle, u64>,
    layers: &BTreeMap<String, u32>,
    patterns: &super::line_pattern::ExportLinePatternMap,
    diagnostics: &mut Vec<CadToOcdrawDiagnostic>,
) -> Result<BTreeSet<Handle>, crate::CadToOcdrawError> {
    let mut consumed = Vec::new();
    for section in cad_override_sections(source) {
        let location = format!(
            "object/{}.viewport/{}.layer/{}",
            section.record, section.viewport, section.layer
        );
        let viewport = entities
            .get(&section.viewport)
            .and_then(|id| drawing.viewports.iter_mut().find(|v| v.id == *id));
        let layer_id = source
            .layers
            .iter()
            .find(|l| l.handle == section.layer)
            .and_then(|l| layers.get(&l.name.to_lowercase()))
            .copied();
        let mut represented = false;
        if let (Some(viewport), Some(layer_id)) = (viewport, layer_id) {
            let mut row = viewport
                .layer_overrides
                .iter()
                .find(|r| r.layer_id == layer_id)
                .cloned()
                .unwrap_or(DrawingViewportLayerOverride {
                    layer_id,
                    frozen: false,
                    color: None,
                    opacity: None,
                    line_pattern_id: None,
                    line_weight: None,
                });
            let mut conflict = false;
            let value = match section.kind {
                K::LayerViewportColorOverride => decode_override_color(&section.value).map(|v| {
                    let v = DrawingColor {
                        rgb: v.rgb,
                        indexed: v.indexed,
                        named: v.named,
                    };
                    conflict = row.color.as_ref().is_some_and(|old| *old != v);
                    row.color = Some(v);
                }),
                K::LayerViewportAlphaOverride => decode_override_opacity(&section.value).map(|v| {
                    conflict = row.opacity.is_some_and(|old| old != v);
                    row.opacity = Some(v);
                }),
                K::LayerViewportLineweightOverride => decode_override_lineweight(&section.value)
                    .map(|v| {
                        conflict = row.line_weight.is_some_and(|old| old != v);
                        row.line_weight = Some(v);
                    }),
                K::LayerViewportLinetypeOverride => {
                    let pattern = if let XRecordValue::Handle(h) = section.value {
                        source
                            .line_types
                            .iter()
                            .find(|p| p.handle == h)
                            .map(|p| patterns.resolve_preserved(&p.name, Some(h)))
                            .transpose()?
                            .flatten()
                    } else {
                        None
                    };
                    if let Some(AppearanceSelection::Explicit(v)) = pattern {
                        conflict = row.line_pattern_id.is_some_and(|old| old != v);
                        row.line_pattern_id = Some(v);
                        Ok(())
                    } else {
                        Err(PresentationValueError::UnsupportedOverride)
                    }
                }
                _ => unreachable!(),
            };
            if conflict {
                return Err(crate::CadToOcdrawError::InvalidSourceStructure {
                    problems: vec![
                        crate::source::CadSourceStructureProblem::InconsistentRelationship {
                            description: format!("conflicting viewport override at {location}"),
                        },
                    ],
                });
            }
            if value.is_ok() {
                if let Some(existing) = viewport
                    .layer_overrides
                    .iter_mut()
                    .find(|r| r.layer_id == layer_id)
                {
                    *existing = row;
                } else {
                    viewport.layer_overrides.push(row);
                }
                viewport.layer_overrides.sort_by_key(|r| r.layer_id);
                represented = true;
            }
        }
        if represented {
            consumed.push(section);
        } else {
            diagnostics.push(CadToOcdrawDiagnostic::loss(
                CadToOcdrawDiagnosticSource::DocumentField { name: location },
                CadToOcdrawAction::PartiallyExported,
                vec![CadToOcdrawLossReason::UnsupportedSemantic {
                    name: format!(
                        "unqualified {:?} or unavailable viewport/layer/pattern",
                        section.kind
                    ),
                }],
            ));
        }
    }
    Ok(consumed_override_objects(source, &consumed))
}

pub(crate) fn to_cad(
    drawing: &OcdrawDocument,
    document: &mut CadDocument,
    entities: &BTreeMap<u64, Handle>,
    layers: &BTreeMap<u64, String>,
    patterns: &super::line_pattern::ImportLinePatternMap,
    diagnostics: &mut Vec<crate::OcdrawToCadDiagnostic>,
) {
    for view in &drawing.viewports {
        let Some(viewport) = entities.get(&view.id).copied() else {
            continue;
        };
        for row in &view.layer_overrides {
            let location = format!("/entities/{}/layerOverrides/{}", view.id, row.layer_id);
            let Some(layer) = layers
                .get(&u64::from(row.layer_id))
                .and_then(|name| document.layers.get(name))
                .map(|l| l.handle)
            else {
                diagnostics.push(crate::to_cad::diagnostic(
                    "VIEWPORT_OVERRIDE",
                    location,
                    "unavailable target layer",
                ));
                continue;
            };
            let mut values = Vec::new();
            if let Some(color) = &row.color {
                let color = CadColorValue {
                    rgb: color.rgb,
                    indexed: color.indexed.clone(),
                    named: color.named.clone(),
                };
                let (value, losses) = encode_override_color(&color).expect("validated color");
                for loss in losses {
                    diagnostics.push(crate::to_cad::diagnostic(
                        "VIEWPORT_OVERRIDE",
                        format!("{location}.color"),
                        format!("indexed identity omitted: {loss:?}"),
                    ));
                }
                if color.named.is_some() {
                    diagnostics.push(crate::to_cad::diagnostic(
                        "VIEWPORT_OVERRIDE",
                        format!("{location}.color.named"),
                        "named viewport color metadata is not qualified; concrete color retained",
                    ));
                }
                values.push((K::LayerViewportColorOverride, value));
            }
            if let Some(value) = row.opacity {
                let value = crate::to_cad::opacity(value, &location, diagnostics);
                values.push((
                    K::LayerViewportAlphaOverride,
                    encode_override_opacity(value).expect("explicit opacity"),
                ));
            }
            if let Some(value) = row.line_weight {
                let value = crate::to_cad::line_weight(value, &location, diagnostics);
                values.push((
                    K::LayerViewportLineweightOverride,
                    encode_override_lineweight(value).expect("explicit weight"),
                ));
            }
            if let Some(id) = row.line_pattern_id {
                if let Some((_, handle)) = patterns.get(&id) {
                    values.push((
                        K::LayerViewportLinetypeOverride,
                        XRecordValue::Handle(*handle),
                    ));
                } else {
                    diagnostics.push(crate::to_cad::diagnostic(
                        "VIEWPORT_OVERRIDE",
                        format!("{location}.linePattern"),
                        "unavailable target pattern",
                    ));
                }
            }
            for (kind, value) in values {
                assert!(document.set_layer_viewport_override(layer, kind, viewport, value));
            }
        }
    }
}
