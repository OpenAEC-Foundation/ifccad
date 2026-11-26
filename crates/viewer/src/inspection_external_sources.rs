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
    json!(d.model.entities.iter().filter_map(IfccadEntity::as_native).filter_map(|e|{
        let IfccadEntityKind::Xref(x)=&e.kind else{return None};
        Some(json!({"entityId":e.id.to_string(),"entityPath":format!("/cad/d{}/e{}",d.drawing_id,e.id),
            "sourceId":x.source_id.0.to_string(),"sourcePath":format!("/cad/d{}/externalSource/{}",d.drawing_id,x.source_id.0),
            "placement":x.placement,"scale":x.scale}))
    }).collect::<Vec<_>>())
}
