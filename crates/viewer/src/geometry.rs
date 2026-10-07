//! Conversion evidence presentation; uint64 identities are decimal strings.
use ifccad_convert::*;
use serde_json::{json, Value};
fn domain(d: IfccadGeometryDomain) -> Value {
    match d {
        IfccadGeometryDomain::Drawing => json!({"kind":"Drawing"}),
        IfccadGeometryDomain::PaperLayout(id) => {
            json!({"kind":"PaperLayout","layoutId":id.to_string()})
        }
    }
}
fn owner(o: IfccadGeometryOwner) -> Value {
    let (kind, id) = match o {
        IfccadGeometryOwner::ModelLayout(id) => ("ModelLayout", id),
        IfccadGeometryOwner::PaperLayout(id) => ("PaperLayout", id),
        IfccadGeometryOwner::BlockDefinition(id) => ("BlockDefinition", id),
    };
    json!({"kind":kind,"id":id.to_string()})
}
fn source(s: &IfccadGeometryEntitySource) -> Value {
    match s {
        IfccadGeometryEntitySource::CadEntity { handle, kind } => {
            json!({"kind":"CadEntity","handle":handle.value().to_string(),"entityKind":kind})
        }
        IfccadGeometryEntitySource::NativeEntity {
            owner: o,
            entity_id,
        } => json!({"kind":"NativeEntity","owner":owner(*o),"entityId":entity_id.to_string()}),
        IfccadGeometryEntitySource::BlockOccurrence { path, leaf } => {
            json!({"kind":"BlockOccurrence","path":path.iter().map(source).collect::<Vec<_>>(),"leaf":source(leaf)})
        }
    }
}
fn interval(d: IfccadDistanceInterval) -> Value {
    json!({"lower":d.lower(),"upper":d.upper()})
}
pub(crate) fn assessment(a: &IfccadGeometryAssessment) -> Value {
    json!({"complete":a.is_complete(),"unassessedSources":a.unassessed_entities().iter().map(source).collect::<Vec<_>>(),"status":format!("{:?}",a.status()),"domains":a.domains().iter().map(|d|json!({
        "domain":domain(d.domain()),"coordinateMeaning":meaning(d.coordinate_meaning()),"status":format!("{:?}",d.status()),
        "requestedTolerance":format!("{:?}",d.requested_tolerance()),"resolvedTolerance":interval(d.resolved_tolerance()),
        "assessedEntities":d.assessed_entities(),"assessedVertices":d.assessed_vertices(),"roundedEntities":d.rounded_entities(),
        "maxDeviationUpperBound":d.max_deviation_upper_bound(),"worstEntity":d.worst_entity().map(source)
    })).collect::<Vec<_>>()})
}
pub(crate) fn failure(output: &mut Value, error: &IfccadConversionError) {
    if let IfccadConversionError::Geometry(f) = error {
        output["failure"]["geometry"] = json!({"domain":domain(f.domain),"coordinateMeaning":meaning(&f.coordinate_meaning),"source":source(&f.source),
            "stage":format!("{:?}",f.stage),"reason":format!("{:?}",f.reason),"vertexIndex":f.vertex_index,
            "requestedTolerance":format!("{:?}",f.requested_tolerance),"resolvedTolerance":f.resolved_tolerance.map(interval),"deviation":f.deviation.map(interval)});
    } else if let IfccadConversionError::PaperTolerance { layout_id, reason } = error {
        output["failure"]["geometry"] = json!({"domain":{"kind":"PaperLayout","layoutId":layout_id.to_string()},"reason":format!("{reason:?}")});
    } else if let IfccadConversionError::Tolerance(reason) = error {
        output["failure"]["geometry"] = json!({"reason":format!("{reason:?}")});
    }
}

pub(crate) fn meaning(m: &IfccadGeometryCoordinateMeaning) -> Value {
    match m {
        IfccadGeometryCoordinateMeaning::DrawingUnit(u) => {
            json!({"kind":"DrawingUnit","unit":u.as_str()})
        }
        IfccadGeometryCoordinateMeaning::PaperCoordinates { mapping } => {
            json!({"kind":"PaperCoordinates","physicalOutputFactor":match mapping {
                IfccadPaperMapping::Unknown=>Value::Null,
                IfccadPaperMapping::FixedPhysical{metres_per_coordinate:q}=>json!({"numerator":q.numer().to_string(),"denominator":q.denom().to_string()}),
            }})
        }
    }
}
