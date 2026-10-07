use ocdraw::ifccad::*;
use serde::{Deserialize, Serialize};
pub(crate) const ENTITY_PREDICATE: &str = "openaec.ifccad.splineEntityBinding";
pub(crate) const COORDINATE_PREDICATE: &str = "openaec.ifccad.splineCoordinateContext";
pub(crate) const REFERENCE_PREDICATE: &str = "openaec.ifccad.sourceReferenceBinding";
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct EntityBaseline {
    pub preservation_record_path: String,
    pub owner_path: String,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CoordinateBaseline {
    pub kind: String,
    pub meaning: Meaning,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase", deny_unknown_fields)]
pub(crate) enum Meaning {
    DrawingUnit {
        unit: String,
    },
    Block {
        unit: String,
        #[serde(rename = "insertionUnit")]
        insertion_unit: String,
    },
    Unknown,
    FixedPhysical {
        numerator: String,
        denominator: String,
    },
}
pub(crate) fn owner(
    d: &IfccadDocument,
    id: u64,
) -> Option<(IfccadPreservationTarget, String, CoordinateBaseline)> {
    let prefix = format!("/cad/d{}", d.drawing_id);
    if d.model.entities.iter().any(|e| e.id() == id) {
        return Some((
            IfccadPreservationTarget::Layout(d.model.id),
            format!("{prefix}/layout/{}", d.model.id),
            CoordinateBaseline {
                kind: "Model".into(),
                meaning: Meaning::DrawingUnit {
                    unit: d.length_unit.clone(),
                },
            },
        ));
    }
    for p in &d.paper_layouts {
        if p.entities.iter().any(|e| e.id() == id) {
            let meaning = match cad_geometry_convert::plot_units::paper_mapping(
                p.settings.plot_settings.as_ref(),
            ) {
                cad_geometry_convert::plot_units::PaperMapping::Unknown => Meaning::Unknown,
                cad_geometry_convert::plot_units::PaperMapping::FixedPhysical {
                    metres_per_coordinate: q,
                } => Meaning::FixedPhysical {
                    numerator: q.numer().to_string(),
                    denominator: q.denom().to_string(),
                },
            };
            return Some((
                IfccadPreservationTarget::Layout(p.id),
                format!("{prefix}/layout/{}", p.id),
                CoordinateBaseline {
                    kind: "Paper".into(),
                    meaning,
                },
            ));
        }
    }
    d.blocks
        .iter()
        .find(|b| b.entities.iter().any(|e| e.id() == id))
        .map(|b| {
            (
                IfccadPreservationTarget::BlockDefinition(b.id),
                format!("{prefix}/block/{}", b.id),
                CoordinateBaseline {
                    kind: "Block".into(),
                    meaning: Meaning::Block {
                        unit: d.length_unit.clone(),
                        insertion_unit: b.insertion_unit.clone(),
                    },
                },
            )
        })
}
pub(crate) fn build(
    d: &IfccadDocument,
    id: u64,
    record: IfccadPreservationRecordId,
) -> Vec<IfccadPreservationCondition> {
    let (target, path, coordinate) = owner(d, id).expect("captured entity has checked ownership");
    vec![
        IfccadPreservationCondition {
            target: IfccadPreservationTarget::Entity(id),
            predicate: ENTITY_PREDICATE.into(),
            version: 1,
            baseline: serde_json::to_vec(&EntityBaseline {
                preservation_record_path: format!(
                    "/cad/d{}/preservation/r{}",
                    d.drawing_id, record.0
                ),
                owner_path: path,
            })
            .unwrap(),
        },
        IfccadPreservationCondition {
            target,
            predicate: COORDINATE_PREDICATE.into(),
            version: 1,
            baseline: serde_json::to_vec(&coordinate).unwrap(),
        },
    ]
}

pub(crate) fn evaluate(
    r: &IfccadPreservationRecord,
    e: &IfccadOpaqueEntity,
    d: &IfccadDocument,
) -> Result<(), super::IfccadPreservationReason> {
    use super::IfccadPreservationReason::*;
    let (target, path, actual) = owner(d, e.id).ok_or(MissingDependency)?;
    let mut entity = false;
    let mut coordinates = false;
    let mut references = std::collections::BTreeSet::new();
    for c in &r.conditions {
        if c.version != 1 {
            return Err(UnsupportedPredicate);
        }
        match c.predicate.as_str() {
            ENTITY_PREDICATE => {
                if entity || c.target != IfccadPreservationTarget::Entity(e.id) {
                    return Err(UnsupportedContext);
                }
                entity = true;
                let stored: EntityBaseline =
                    serde_json::from_slice(&c.baseline).map_err(|_| MalformedPayload)?;
                if stored.preservation_record_path
                    != format!("/cad/d{}/preservation/r{}", d.drawing_id, r.id.0)
                    || stored.owner_path != path
                {
                    return Err(ChangedDependency);
                }
            }
            COORDINATE_PREDICATE => {
                if coordinates || c.target != target {
                    return Err(UnsupportedContext);
                }
                coordinates = true;
                let stored: CoordinateBaseline =
                    serde_json::from_slice(&c.baseline).map_err(|_| MalformedPayload)?;
                if stored != actual {
                    return Err(ChangedDependency);
                }
            }
            REFERENCE_PREDICATE => {
                super::references::validate_reference_condition(r, c)?;
                let stored: super::references::ReferenceBaseline =
                    serde_json::from_slice(&c.baseline).map_err(|_| MalformedPayload)?;
                if !references.insert(stored.slot) {
                    return Err(UnsupportedContext);
                }
            }
            _ => return Err(UnsupportedPredicate),
        }
    }
    if !entity || !coordinates {
        return Err(MissingDependency);
    }
    Ok(())
}
pub(crate) fn unsupported_common_context(spline: &opencadcodec::entities::Spline) -> bool {
    let c = &spline.common;
    c.graphic_data.is_some()
        || !c.extended_data.raw_dwg_eed.is_empty()
        || !c.reactors.is_empty()
        || c.xdictionary_handle.is_some()
        || c.color_book_handle.is_some()
        || c.full_visual_style_handle.is_some()
        || c.face_visual_style_handle.is_some()
        || c.edge_visual_style_handle.is_some()
        || c.material_flags != 0
        || c.material_handle.is_some()
        || c.shadow_flags != 0
        || c.plotstyle_flags != 0
        || c.plotstyle_handle.is_some()
        || c.has_ds_data
        || c.extended_data
            .records()
            .iter()
            .flat_map(|r| &r.values)
            .any(|v| {
                matches!(
                    v,
                    opencadcodec::xdata::XDataValue::String(_)
                        | opencadcodec::xdata::XDataValue::ControlString(_)
                        | opencadcodec::xdata::XDataValue::BinaryData(_)
                )
            })
}
