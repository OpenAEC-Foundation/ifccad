use crate::{invalid, unsupported, CadDimensionPreparationError};
use ocdraw::dimension::{DimensionPointSelector, DimensionRole};
use opencadcodec::Handle;
use opencadcodec::{objects::AssocDimensionAssociation, CadDocument, EntityType};
#[derive(Clone, Debug)]
pub struct CadDimensionPointReference {
    pub role: DimensionRole,
    pub source: Handle,
    pub selector: DimensionPointSelector,
}

/// Qualifies ordinary single-object snaps; it never evaluates a dimension or guesses a nearest point.
pub fn prepare_point_references(
    a: &AssocDimensionAssociation,
    doc: &CadDocument,
) -> Result<Vec<CadDimensionPointReference>, CadDimensionPreparationError> {
    if a.trans_space || a.rotated_type != 0 || a.associativity & !3 != 0 {
        return Err(unsupported(
            "associations",
            "trans-space, nonparallel or third/fourth role",
        ));
    }
    let mut result = Vec::new();
    for (slot, refs) in a.references.iter().enumerate() {
        let active = a.associativity & (1 << slot) != 0;
        if !active {
            if !refs.is_empty() {
                return Err(invalid("associations", "inactive slot contains references"));
            }
            continue;
        }
        if refs.len() != 1 {
            return Err(unsupported(
                "associations",
                "missing or chained point reference",
            ));
        }
        let r = &refs[0];
        if r.class_name != "AcDbOsnapPointRef"
            || r.xrefs.len() != 1
            || !r.xref_paths.is_empty()
            || !r.intersection_objects.is_empty()
            || !r.intersection_xref_paths.is_empty()
            || r.has_last_point_reference
            || r.main_subent_type != 0
            || r.main_gs_marker != 0
            || r.intersection_subent_type != 0
            || r.intersection_gs_marker != 0
            || r.osnap_distance != 0.0
        {
            return Err(unsupported(
                "associations/reference",
                "subentity, intersection, path or parameter state",
            ));
        }
        if ![r.osnap_point.x, r.osnap_point.y, r.osnap_point.z]
            .into_iter()
            .all(f64::is_finite)
        {
            return Err(invalid("associations/osnapPoint", "nonfinite point"));
        }
        let h = r.xrefs[0];
        let source = doc
            .get_entity(h)
            .ok_or_else(|| invalid("associations/source", "dangling source handle"))?;
        let selector = match (r.osnap_type, source) {
            (13, EntityType::Line(_)) => DimensionPointSelector::Start,
            (1, EntityType::Line(line)) => {
                let first = r.osnap_point == line.start;
                let second = r.osnap_point == line.end;
                match (first, second) {
                    (true, false) => DimensionPointSelector::Start,
                    (false, true) => DimensionPointSelector::End,
                    _ => {
                        return Err(unsupported(
                            "associations/endpoint",
                            "stored point does not identify one unique endpoint",
                        ))
                    }
                }
            }
            (3, EntityType::Circle(_)) => DimensionPointSelector::Center,
            (4, EntityType::Point(_)) => DimensionPointSelector::Position,
            _ => {
                return Err(unsupported(
                    "associations/selector",
                    "unqualified snap/source pair",
                ))
            }
        };
        result.push(CadDimensionPointReference {
            role: if slot == 0 {
                DimensionRole::FirstPoint
            } else {
                DimensionRole::SecondPoint
            },
            source: h,
            selector,
        });
    }
    Ok(result)
}

/// Binds already allocated CAD handles without changing the stored dimension definition.
pub fn encode_point_references(
    dimension: Handle,
    refs: &[CadDimensionPointReference],
    doc: &CadDocument,
) -> Result<AssocDimensionAssociation, CadDimensionPreparationError> {
    use opencadcodec::objects::AssocDimensionReference;
    let target = doc
        .get_entity(dimension)
        .ok_or_else(|| invalid("associations/dimension", "unallocated dimension handle"))?;
    let mut a = AssocDimensionAssociation {
        dimension,
        ..Default::default()
    };
    for r in refs {
        let slot = match r.role {
            DimensionRole::FirstPoint => 0,
            DimensionRole::SecondPoint => 1,
        };
        if !a.references[slot].is_empty() {
            return Err(invalid("associations/role", "duplicate point role"));
        }
        let source = doc
            .get_entity(r.source)
            .ok_or_else(|| unsupported("associations/source", "source entity was not converted"))?;
        if source.common().owner_handle != target.common().owner_handle {
            return Err(unsupported("associations/source", "cross-owner reference"));
        }
        let (osnap_type, osnap_point) = match (&r.selector, source) {
            (DimensionPointSelector::Start, EntityType::Line(l)) => (13, l.start),
            (DimensionPointSelector::End, EntityType::Line(l)) if l.start != l.end => (1, l.end),
            (DimensionPointSelector::Center, EntityType::Circle(c)) => {
                let frame =
                    cad_geometry_convert::geometry::circular::from_cad_ocs(c.center, c.normal)
                        .ok_or_else(|| {
                            crate::numeric(
                                "associations/osnapPoint",
                                "circle center cannot be represented in WCS",
                            )
                        })?;
                let p = frame.origin();
                (3, opencadcodec::Vector3::new(p.x(), p.y(), p.z()))
            }
            (DimensionPointSelector::Position, EntityType::Point(p)) => (4, p.location),
            _ => {
                return Err(unsupported(
                    "associations/selector",
                    "target snap/source pair is unqualified",
                ))
            }
        };
        a.associativity |= 1 << slot;
        a.references[slot].push(AssocDimensionReference {
            class_name: "AcDbOsnapPointRef".into(),
            osnap_type,
            xrefs: vec![r.source],
            osnap_point,
            ..Default::default()
        });
    }
    Ok(a)
}
