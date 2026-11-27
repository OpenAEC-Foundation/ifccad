//! Authored external identities and placements; inspection performs no lookup.
use ifccad::{IfccadDocument, IfccadEntity, IfccadEntityKind};
use serde_json::{json, Value};

pub(crate) fn sources(d: &IfccadDocument) -> Value {
    json!(d.external_sources.iter().map(|s|json!({
        "id":s.id.0.to_string(),"sourcePath":format!("/cad/d{}/externalSource/{}",d.drawing_id,s.id.0),
        "name":s.name,"location":s.location,"source":s.source,"status":"notChecked"
    })).collect::<Vec<_>>())
}

pub(crate) fn instances(d: &IfccadDocument) -> Value {
    let root = format!("/cad/d{}", d.drawing_id);
    let scopes = std::iter::once((
        format!("{root}/layout/{}", d.model.id),
        "model",
        d.model.entities.as_slice(),
    ))
    .chain(d.paper_layouts.iter().map(|p| {
        (
            format!("{root}/layout/{}", p.id),
            "paper",
            p.entities.as_slice(),
        )
    }))
    .chain(d.blocks.iter().map(|b| {
        (
            format!("{root}/block/{}", b.id),
            "block",
            b.entities.as_slice(),
        )
    }));
    json!(scopes.flat_map(|(owner_path,owner_kind,entities)|entities.iter().filter_map(IfccadEntity::as_native).filter_map(move|e|{
        let IfccadEntityKind::Xref(x)=&e.kind else{return None};
        Some(json!({"entityId":e.id.to_string(),"entityPath":format!("/cad/d{}/e{}",d.drawing_id,e.id),
            "ownerPath":owner_path,"ownerKind":owner_kind,
            "sourceId":x.source_id.0.to_string(),"sourcePath":format!("/cad/d{}/externalSource/{}",d.drawing_id,x.source_id.0),
            "transform":x.transform,"referenceType":x.reference_type}))
    })).collect::<Vec<_>>())
}
