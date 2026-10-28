//! Core validation and source inspection, independent of CAD conversion.
use crate::{fail, result};
use ifccad::{load_ifccad_bytes, IfccadDocument};
use serde_json::{json, Value};
use std::path::Path;

/// Core inspection only: never prepare a reference CAD conversion.
pub(crate) fn present_document(output: &mut Value, d: &IfccadDocument, graph: &Value) {
    let nodes = graph["data"].as_array().expect("validated graph");
    let role = |key: &str| {
        nodes
            .iter()
            .filter(|n| n["attributes"].get(key).is_some())
            .cloned()
            .collect::<Vec<_>>()
    };
    use ifccad::{assess_ifccad_document_bounds, IfccadBoundsQuality as Q, IfccadScopeId as S};
    let completeness=assess_ifccad_document_bounds(d).expect("validated numerical extents").scopes.into_iter().map(|(scope,a)|{
        let (scope_path,bounds,claim)=match scope {
            S::Layout(id) if id==d.model.id=>(format!("/cad/d{}/layout/{id}",d.drawing_id),d.model.bounds,d.model.bounds_quality),
            S::Layout(id)=>{let p=d.paper_layouts.iter().find(|p|p.id==id).unwrap();(format!("/cad/d{}/layout/{id}",d.drawing_id),p.bounds,p.bounds_quality)},
            S::BlockDefinition(id)=>{let b=d.blocks.iter().find(|b|b.id==id).unwrap();(format!("/cad/d{}/block/{id}",d.drawing_id),b.bounds,b.bounds_quality)},
        };
        let quality=|q:Q|match q {Q::Enclosing=>"enclosing",Q::Estimated=>"estimated",Q::Partial=>"partial"};
        json!({"scope":format!("{scope:?}"),"scopePath":scope_path,"complete":a.quality!=Some(Q::Partial),"state":if a.quality==Some(Q::Partial){"Unavailable"}else if a.bounds.is_none(){"Empty"}else{"Complete"},"quality":a.quality.map(quality),"bounds":a.bounds,"storedBounds":bounds,"storedQuality":bounds.map(|_|quality(claim.unwrap_or(Q::Enclosing))),"enclosureVerified":a.enclosure_verified,"safeForNegativeQuery":a.enclosure_verified&&bounds.is_some()&&claim!=Some(Q::Estimated),"textReasons":a.text_reasons.iter().map(|r|format!("{r:?}")).collect::<Vec<_>>(),"polylineWidthReasons":a.polyline_width_reasons.iter().map(|r|json!({"sourcePath":r.source_path,"occurrencePath":r.occurrence_path.iter().map(u64::to_string).collect::<Vec<_>>(),"reason":format!("{:?}",r.reason)})).collect::<Vec<_>>(),"linePatternReasons":a.line_pattern_reasons.iter().map(|r|format!("{r:?}")).collect::<Vec<_>>()})
    }).collect::<Vec<_>>();
    let hatches = role("ifccad::hatch")
        .into_iter()
        .map(|mut node| {
            let body = &node["attributes"]["ifccad::hatch"];
            let status = gradient_status(&body["fill"], &body["gradient"]);
            let association_status = if body["associationEnabled"] != true {
                "disabled"
            } else if body["loops"].as_array().is_some_and(|loops| {
                loops
                    .iter()
                    .any(|l| l["sources"].as_array().is_some_and(|ids| !ids.is_empty()))
            }) {
                "active"
            } else {
                "unbound"
            };
            node["gradientStatus"] = json!(status);
            node["associationStatus"] = json!(association_status);
            node
        })
        .collect::<Vec<_>>();
    output["validation"] = json!({"strictAvailable":true,"status":"valid","diagnostics":[]});
    output["presentation"] = json!({"format":"ifccad","drawingId":d.drawing_id,"unit":d.length_unit,"linePatternScale":d.line_pattern_scale,"pointDisplay":nodes.iter().find_map(|n|n["attributes"].get("ifccad::drawing")).and_then(|d|d.get("pointDisplay")),
        "layers":role("ifccad::layer"),"layouts":role("ifccad::layout"),"blockDefinitions":role("ifccad::blockDefinition"),
        "linePatterns":role("ifccad::linePattern"),"symbolSources":role("ifccad::symbolSource"),"ucsDefinitions":role("ifccad::ucsDefinition"),"modelWindows":role("ifccad::modelWindow"),"hatchEntityCount":role("ifccad::hatch").len(),"hatches":hatches,"hatchFillEvaluation":"unassessed","textStyles":role("ifccad::textStyle"),"textEntityCount":role("ifccad::text").len(),"mTextEntityCount":role("ifccad::mText").len(),"entities":role("ifccad::entity"),"opaqueEntities":role("ifccad::opaqueEntity"),"opaqueEntityCount":role("ifccad::opaqueEntity").len(),"preservationRecords":role("ifccad::preservationRecord"),"preservationSources":d.preservation.as_ref().map(|p|&p.sources),"boundsCompleteness":completeness,"graph":graph});
    output["presentation"]["polylineWidths"] = crate::inspection_polyline_widths::authored(graph);
    output["presentation"]["dimensions"] = crate::inspection_ifccad_dimension::entities(d);
    output["presentation"]["dimensionStyles"] = crate::inspection_ifccad_dimension::styles(d);
}
pub fn inspect_ifccad_bytes(name: &str, bytes: &[u8]) -> Value {
    if name.to_ascii_lowercase().ends_with(".ocdraw.json")
        || name.to_ascii_lowercase().ends_with(".ocdraw")
    {
        let mut output = result(Path::new(name), "ocdraw");
        fail(
            &mut output,
            "reading",
            "OCDRAW_PARKED",
            "OCDraw is parked; use the archived reference or choose IFCCAD.",
        );
        return output;
    }

    if crate::ifccad_input::encoding(name) != "ifcx" {
        return crate::ifccad_input::inspect(name, bytes);
    }
    let mut output = result(Path::new(name), "ifccad");
    match load_ifccad_bytes(bytes, Default::default()) {
        Ok(drawing) => {
            present_document(
                &mut output,
                drawing.document(),
                drawing.graph().composed_ifcx(),
            );
        }
        Err(report) => {
            output["validation"] = json!({"strictAvailable":false,"status":"invalid","diagnostics":report.report().errors.iter().map(|e|json!({"code":"IFCCAD_PROFILE","message":e})).collect::<Vec<_>>()});
            fail(
                &mut output,
                "validating",
                "IFCCAD_PROFILE_INVALID",
                report.report().errors.join("; "),
            );
        }
    }
    output
}

pub(crate) fn gradient_status(fill: &Value, gradient: &Value) -> &'static str {
    if gradient.is_null() {
        "absent"
    } else if fill["kind"] == "gradient" {
        "active"
    } else if gradient.get("type").is_some() && gradient.get("colors").is_some() {
        "inactive"
    } else {
        "partial"
    }
}
