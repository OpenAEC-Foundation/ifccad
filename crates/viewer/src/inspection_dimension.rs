//! Typed OCDraw dimension inspection; the formatter never follows source associations.
use ocdraw::{dimension::*, ocdraw::*};
use serde_json::{json, Value};
fn color(c: &DrawingColor) -> Value {
    match c {
        DrawingColor::Foreground => json!({"kind":"Foreground"}),
        DrawingColor::Concrete {
            rgb,
            indexed,
            named,
        } => json!({"rgb":rgb,"indexed":indexed,"named":named}),
    }
}
fn settings(v: &OcdrawDimensionSettings) -> Value {
    serde_json::to_value(v.map_references(
        |id| id.0.to_string(),
        u32::to_string,
        color,
        |id| id.0.to_string(),
    ))
    .expect("serializable typed settings")
}
fn overrides(v: &OcdrawDimensionOverrides) -> Value {
    serde_json::to_value(v.map_references(
        |id| id.0.to_string(),
        u32::to_string,
        color,
        |id| id.0.to_string(),
    ))
    .expect("serializable typed overrides")
}
pub(crate) fn styles(doc: &OcdrawDocument) -> Value {
    json!(doc.dimension_styles.iter().map(|s|json!({"id":s.id.0.to_string(),"name":s.name,"settings":settings(&s.settings),"typeOverrides":s.type_overrides.iter().map(|(k,v)|(format!("{k:?}"),overrides(v))).collect::<serde_json::Map<_,_>>()})).collect::<Vec<_>>())
}
pub(crate) fn entities(doc: &OcdrawDocument) -> Vec<Value> {
    doc.dimension_entities.iter().map(|d|{
        let style=doc.dimension_styles.iter().find(|s|s.id==d.style_id).expect("validated style");
        let effective=resolve_dimension_settings(&style.settings,d.definition.kind(),&style.type_overrides,&d.overrides);
        let is_paper=doc.layouts.iter().any(|l|l.kind==DrawingLayoutKind::Paper&&doc.scopes.iter().any(|s|s.id==l.scope_id&&s.entities.contains(&d.id)));
        let measurement=evaluate_dimension(&d.definition).expect("validated measurement");
        let raw=match measurement{DimensionMeasurement::Distance(v)=>json!({"kind":"distance","value":v}),DimensionMeasurement::Angle(v)=>json!({"kind":"angle","value":v})};
        let label=match format_dimension_label(measurement,&effective.numbers,DimensionFormatContext{is_paper},&d.label){Ok(v)=>crate::inspection_dimension_label::label(v),Err(e)=>json!({"state":"unavailable","reason":e.to_string()})};
        json!({"id":d.id.to_string(),"layerId":d.layer_id,"styleId":d.style_id.0.to_string(),"visible":d.visible,"appearance":crate::inspection::appearance(&d.appearance),"geometry":{"type":"dimension","variant":d.definition.kind(),"definition":d.definition,"placement":{"origin":d.placement.origin().components(),"xAxis":d.placement.x_axis().components(),"yAxis":d.placement.y_axis().components()},"textPlacement":d.text_placement,"labelDefinition":d.label,"rawMeasurement":raw,"label":label,"effectiveSettings":settings(&effective),"overrides":overrides(&d.overrides),"associations":d.associations.iter().map(|r|json!({"role":r.role,"sourceEntityId":r.source_entity_id.to_string(),"selector":r.selector})).collect::<Vec<_>>(),"associationUpdate":"external","displayEvaluation":"unassessed"}})
    }).collect()
}
pub(crate) fn report(r: &ocdraw_convert::OcdrawDimensionAssessment) -> Value {
    use ocdraw_convert::OcdrawDimensionNumericCoverage as N;
    json!({"entries":r.entries().iter().map(|e|json!({"entityId":e.entity_id.to_string(),"numericCoverage":match e.numeric_coverage{N::StoredDefinitionAnchors=>"storedDefinitionAnchors",N::StoredDefinitionAndExplicitTextAnchors=>"storedDefinitionAndExplicitTextAnchors",N::NotTransferred=>"notTransferred"},"displayCoverage":"unassessed"})).collect::<Vec<_>>()})
}
