//! Independent typed IFCCAD dimension inspection; full-width IDs become complete paths.
use ifccad::{dimension::*, *};
use serde_json::{json, Value};
fn color(c: &IfccadColor) -> Value {
    match c {
        IfccadColor::Foreground => json!({"kind":"Foreground"}),
        IfccadColor::Concrete {
            rgb,
            indexed,
            named,
        } => json!({"rgb":rgb,"indexed":indexed,"named":named}),
    }
}
fn settings(v: &IfccadDimensionSettings, id: u64) -> Value {
    serde_json::to_value(v.map_references(
        |v| format!("/cad/d{id}/textStyle/{}", v.0),
        |v| format!("/cad/d{id}/block/{v}"),
        color,
        |v| format!("/cad/d{id}/linePattern/{}", v.0),
    ))
    .expect("serializable typed settings")
}
fn overrides(v: &IfccadDimensionOverrides, id: u64) -> Value {
    serde_json::to_value(v.map_references(
        |v| format!("/cad/d{id}/textStyle/{}", v.0),
        |v| format!("/cad/d{id}/block/{v}"),
        color,
        |v| format!("/cad/d{id}/linePattern/{}", v.0),
    ))
    .expect("serializable typed overrides")
}
pub(crate) fn styles(doc: &IfccadDocument) -> Value {
    json!(doc.dimension_styles.iter().map(|s|json!({"path":format!("/cad/d{}/dimensionStyle/{}",doc.drawing_id,s.id.0),"name":s.name,"settings":settings(&s.settings,doc.drawing_id),"typeOverrides":s.type_overrides.iter().map(|(k,v)|(format!("{k:?}"),overrides(v,doc.drawing_id))).collect::<serde_json::Map<_,_>>()})).collect::<Vec<_>>())
}
pub(crate) fn entities(doc: &IfccadDocument) -> Value {
    let mut output = Vec::new();
    let scopes = std::iter::once((false, &doc.model.entities))
        .chain(doc.paper_layouts.iter().map(|p| (true, &p.entities)))
        .chain(doc.blocks.iter().map(|b| (false, &b.entities)));
    for (is_paper, entities) in scopes {
        for e in entities.iter().filter_map(IfccadEntity::as_native) {
            let IfccadEntityKind::Dimension(d) = &e.kind else {
                continue;
            };
            let style = doc
                .dimension_styles
                .iter()
                .find(|s| s.id == d.style_id)
                .expect("validated style");
            let effective = resolve_dimension_settings(
                &style.settings,
                d.definition.kind(),
                &style.type_overrides,
                &d.overrides,
            );
            let measurement = evaluate_dimension(&d.definition).expect("validated measurement");
            let raw = match measurement {
                DimensionMeasurement::Distance(v) => json!({"kind":"distance","value":v}),
                DimensionMeasurement::Angle(v) => json!({"kind":"angle","value":v}),
            };
            let label = match format_dimension_label(
                measurement,
                &effective.numbers,
                DimensionFormatContext { is_paper },
                &d.label,
            ) {
                Ok(v) => crate::inspection_dimension_label::label(v),
                Err(e) => json!({"state":"unavailable","reason":e.to_string()}),
            };
            output.push(json!({"path":format!("/cad/d{}/e{}",doc.drawing_id,e.id),"entityId":e.id.to_string(),"style":format!("/cad/d{}/dimensionStyle/{}",doc.drawing_id,d.style_id.0),"geometry":{"type":"dimension","variant":d.definition.kind(),"definition":d.definition,"placement":d.placement,"textPlacement":d.text_placement,"rawMeasurement":raw,"labelDefinition":d.label,"label":label,"effectiveSettings":settings(&effective,doc.drawing_id),"overrides":overrides(&d.overrides,doc.drawing_id),"associations":d.associations.iter().map(|r|json!({"role":r.role,"source":format!("/cad/d{}/e{}",doc.drawing_id,r.source_entity_id),"selector":r.selector})).collect::<Vec<_>>(),"associationUpdate":"external","displayEvaluation":"unassessed"}}));
        }
    }
    json!(output)
}
pub(crate) fn report(r: &ifccad_convert::IfccadDimensionAssessment) -> Value {
    use ifccad_convert::IfccadDimensionNumericCoverage as N;
    json!({"entries":r.entries().iter().map(|e|json!({"entityId":e.entity_id.to_string(),"owner":format!("{:?}",e.owner),"numericCoverage":match e.numeric_coverage{N::StoredDefinitionAnchors=>"storedDefinitionAnchors",N::StoredDefinitionAndExplicitTextAnchors=>"storedDefinitionAndExplicitTextAnchors",N::NotTransferred=>"notTransferred"},"displayCoverage":"unassessed"})).collect::<Vec<_>>()})
}
