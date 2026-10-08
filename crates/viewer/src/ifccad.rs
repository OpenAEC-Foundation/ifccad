use crate::{cad, fail, progress, result};
use ifccad_convert::{
    cad_document_to_encoded_ifccad, ifccad_source_to_cad_document, IfccadDiagnostic,
    IfccadTargetMetadata,
};
use ocdraw::ifccad::{load_ifccad_bytes, IfccadHeader, ValidatedIfccad};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn is_ifccad_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.ends_with(".ifcx") || name.ends_with(".ifcx.json")
}
fn diagnostics(items: &[IfccadDiagnostic]) -> Vec<Value> {
    items.iter().map(|d|json!({"code":d.code,"location":d.location,"message":d.message,"action":format!("{:?}",d.action)})).collect()
}
fn text_report(report: &ifccad_convert::IfccadTextAssessment) -> Value {
    use ifccad_convert::{IfccadTextGlyphCoverage as G, IfccadTextNumericCoverage as N};
    json!({"entries":report.entries().iter().map(|e|json!({"entityId":e.entity_id.to_string(),"owner":format!("{:?}",e.owner),"numericCoverage":match e.numeric_coverage{N::ActiveTextAnchors=>"activeTextAnchors",N::MTextWcsAnchor=>"mTextWcsAnchor",N::NotTransferred=>"notTransferred"},"glyphCoverage":match e.glyph_coverage{G::Empty=>"empty",G::Unassessed=>"unassessed"}})).collect::<Vec<_>>()})
}
fn present(output: &mut Value, drawing: &ValidatedIfccad) {
    let d = drawing.document();
    let nodes = drawing.graph().composed_ifcx()["data"]
        .as_array()
        .expect("validated graph");
    let role = |key: &str| {
        nodes
            .iter()
            .filter(|n| n["attributes"].get(key).is_some())
            .cloned()
            .collect::<Vec<_>>()
    };
    use ocdraw::ifccad::{
        assess_ifccad_document_bounds, IfccadBoundsQuality as Q, IfccadScopeId as S,
    };
    let completeness=assess_ifccad_document_bounds(d).expect("validated numerical extents").scopes.into_iter().map(|(scope,a)|{
        let (scope_path,bounds,claim)=match scope {
            S::Layout(id) if id==d.model.id=>(format!("/cad/d{}/layout/{id}",d.drawing_id),d.model.bounds,d.model.bounds_quality),
            S::Layout(id)=>{let p=d.paper_layouts.iter().find(|p|p.id==id).unwrap();(format!("/cad/d{}/layout/{id}",d.drawing_id),p.bounds,p.bounds_quality)},
            S::BlockDefinition(id)=>{let b=d.blocks.iter().find(|b|b.id==id).unwrap();(format!("/cad/d{}/block/{id}",d.drawing_id),b.bounds,b.bounds_quality)},
        };
        let quality=|q:Q|match q {Q::Enclosing=>"enclosing",Q::Estimated=>"estimated",Q::Partial=>"partial"};
        json!({"scope":format!("{scope:?}"),"scopePath":scope_path,"complete":a.quality!=Some(Q::Partial),"state":if a.quality==Some(Q::Partial){"Unavailable"}else if a.bounds.is_none(){"Empty"}else{"Complete"},"quality":a.quality.map(quality),"bounds":a.bounds,"storedBounds":bounds,"storedQuality":bounds.map(|_|quality(claim.unwrap_or(Q::Enclosing))),"enclosureVerified":a.enclosure_verified,"safeForNegativeQuery":a.enclosure_verified&&bounds.is_some()&&claim!=Some(Q::Estimated),"textReasons":a.text_reasons.iter().map(|r|format!("{r:?}")).collect::<Vec<_>>()})
    }).collect::<Vec<_>>();
    output["validation"] = json!({"strictAvailable":true,"status":"valid","diagnostics":[]});
    output["presentation"] = json!({"format":"ifccad","drawingId":d.drawing_id,"unit":d.length_unit,"linePatternScale":d.line_pattern_scale,"pointDisplay":nodes.iter().find_map(|n|n["attributes"].get("ifccad::drawing")).and_then(|d|d.get("pointDisplay")),
        "layers":role("ifccad::layer"),"layouts":role("ifccad::layout"),"blockDefinitions":role("ifccad::blockDefinition"),
        "linePatterns":role("ifccad::linePattern"),"ucsDefinitions":role("ifccad::ucsDefinition"),"modelWindows":role("ifccad::modelWindow"),"hatchEntityCount":role("ifccad::hatch").len(),"hatches":role("ifccad::hatch"),"hatchFillEvaluation":"unassessed","textStyles":role("ifccad::textStyle"),"textEntityCount":role("ifccad::text").len(),"mTextEntityCount":role("ifccad::mText").len(),"entities":role("ifccad::entity"),"opaqueEntities":role("ifccad::opaqueEntity"),"opaqueEntityCount":role("ifccad::opaqueEntity").len(),"preservationRecords":role("ifccad::preservationRecord"),"preservationSources":d.preservation.as_ref().map(|p|&p.sources),"boundsCompleteness":completeness,"graph":drawing.graph().composed_ifcx()});
}
/// Inspect the experimental IFCCAD profile after production composition/validation.
pub fn inspect_ifccad_bytes(name: &str, bytes: &[u8]) -> Value {
    let mut output = result(Path::new(name), "ifccad");
    match load_ifccad_bytes(bytes, Default::default()) {
        Ok(drawing) => present(&mut output, &drawing),
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
/// Convert original CAD bytes directly to IFCCAD; no OCDraw projection is used.
pub fn inspect_cad_as_ifccad_bytes(
    name: &str,
    format: &str,
    bytes: &[u8],
    timestamp: &str,
) -> Value {
    inspect_cad_as_ifccad_with_options(
        name,
        format,
        bytes,
        timestamp,
        &crate::options::ConversionOptions::default(),
    )
}
pub(crate) fn inspect_cad_as_ifccad_with_options(
    name: &str,
    format: &str,
    bytes: &[u8],
    timestamp: &str,
    options: &crate::options::ConversionOptions,
) -> Value {
    let mut output = result(Path::new(name), format);
    let cad = match cad::read(format, bytes) {
        Ok(d) => d,
        Err(error) => {
            fail(&mut output, "reading", "CAD_READ_FAILED", error);
            return output;
        }
    };
    output["reader"]["version"] = json!(cad.dwg_source_version.unwrap_or(cad.version).as_str());
    output["reader"]["messages"] = json!(cad
        .notifications
        .iter()
        .map(|i| format!("{i:?}"))
        .collect::<Vec<_>>());
    let metadata = IfccadTargetMetadata {
        drawing_id: 1,
        header: IfccadHeader {
            id: name.into(),
            data_version: "0.1.0".into(),
            author: "CAD Format Explorer".into(),
            timestamp: timestamp.into(),
        },
    };
    progress("converting");
    let converted = match cad_document_to_encoded_ifccad(
        &cad,
        metadata,
        ifccad_convert::CadToIfccadOptions {
            preservation: if options.preserve_splines {
                ifccad_convert::IfccadPreservationCapture::SupportedTyped
            } else {
                ifccad_convert::IfccadPreservationCapture::Disabled
            },
            geometry_tolerance: options.ocdraw_tolerance().expect("checked options"),
            ..Default::default()
        },
    ) {
        Ok(v) => v,
        Err(error) => {
            fail(
                &mut output,
                "converting",
                "IFCCAD_CONVERSION_FAILED",
                &error,
            );
            crate::geometry::failure(&mut output, &error);
            return output;
        }
    };
    output["conversion"] = json!({"format":"ifccad","preservation":preservation_report(converted.preservation_report()),"diagnostics":diagnostics(converted.diagnostics()),"entityCount":converted.mappings().entities.iter().count(),"options":options.value(),"geometryAssessment":crate::geometry::assessment(converted.geometry_assessment()),"textAssessment":text_report(converted.text_assessment())});
    present(&mut output, converted.validated_source());
    cad::download(&mut output, name, "ifccad", converted.encoded().bytes());
    output
}
/// Preserve original IFCX on native download; project supported content for CAD.
pub fn export_ifccad_bytes(name: &str, bytes: &[u8], format: &str, version: &str) -> Value {
    export_ifccad_with_options(
        name,
        bytes,
        format,
        version,
        &crate::options::ConversionOptions::default(),
    )
}
pub(crate) fn export_ifccad_with_options(
    name: &str,
    bytes: &[u8],
    format: &str,
    version: &str,
    options: &crate::options::ConversionOptions,
) -> Value {
    let mut output = inspect_ifccad_bytes(name, bytes);
    if output["validation"]["strictAvailable"] != true {
        return output;
    }
    if format == "ifccad" {
        cad::download(&mut output, name, "ifccad", bytes);
        return output;
    }
    if let Err((code, message)) = cad::selection(format, version) {
        fail(&mut output, "exporting", code, message);
        return output;
    }
    let drawing = load_ifccad_bytes(bytes, Default::default()).expect("strict status checked");
    progress("converting");
    let converted = match ifccad_source_to_cad_document(
        &drawing,
        ifccad_convert::IfccadToCadOptions {
            geometry_tolerance: options.ocdraw_tolerance().expect("checked options"),
            ..Default::default()
        },
    ) {
        Ok(v) => v,
        Err(error) => {
            fail(&mut output, "converting", "CAD_CONVERSION_FAILED", &error);
            crate::geometry::failure(&mut output, &error);
            return output;
        }
    };
    output["conversion"] = json!({"format":"ifccad","preservation":preservation_report(converted.preservation_report()),"textAssessment":text_report(converted.text_assessment())});
    let issues = diagnostics(converted.diagnostics());
    let assessment = crate::geometry::assessment(converted.geometry_assessment());
    let selection_candidates = crate::selection::ifccad_candidates(
        drawing.document(),
        converted.mappings(),
        converted.document(),
    );
    let mut viewer_selection = Value::Null;
    let metadata = IfccadTargetMetadata {
        header: drawing.document().header.clone(),
        drawing_id: drawing.document().drawing_id,
    };
    let mut readback_geometry = Value::Null;
    let mut output = cad::export(
        output,
        converted.into_document(),
        name,
        format,
        version,
        issues,
        |readback| {
            let restored = cad_document_to_encoded_ifccad(
                readback,
                metadata,
                ifccad_convert::CadToIfccadOptions {
                    preservation: ifccad_convert::IfccadPreservationCapture::SupportedTyped,
                    geometry_tolerance: options.ocdraw_tolerance().expect("checked options"),
                    ..Default::default()
                },
            )
            .map_err(|e| {
                let mut details = json!({"failure":{}});
                crate::geometry::failure(&mut details, &e);
                readback_geometry = details["failure"]["geometry"].clone();
                e.to_string()
            })?;
            load_ifccad_bytes(restored.encoded().bytes(), Default::default())
                .map_err(|e| e.report().errors.join("; "))?;
            viewer_selection =
                crate::selection::qualified_ifccad_selection(&selection_candidates, readback);
            Ok(
                json!({"cadReadback":true,"ifccadStrictReadback":true,"opaqueEntityCount":restored.validated_source().document().preservation.as_ref().map_or(0,|p|p.records.iter().filter(|r|r.subject.is_some()).count()),"preservation":preservation_report(restored.preservation_report()),"diagnostics":diagnostics(restored.diagnostics()),"geometryAssessment":crate::geometry::assessment(restored.geometry_assessment())}),
            )
        },
    );
    output["export"]["geometryAssessment"] = assessment;
    output["export"]["options"] = options.value();
    if output["failure"].is_null() {
        output["export"]["viewerSelection"] = viewer_selection;
    }
    if !readback_geometry.is_null() {
        output["failure"]["geometry"] = readback_geometry;
    }
    output
}

fn preservation_report(report: &ifccad_convert::IfccadPreservationReport) -> Value {
    json!({"entries":report.entries().iter().map(|e|json!({"recordId":e.record_id.map(|id|id.0.to_string()),"entityId":e.entity_id.map(|id|id.to_string()),"sourceId":e.source_id,"sourceKey":e.source_key,"schema":e.schema,"version":e.version,"phase":e.phase,"result":e.result,"reason":e.reason,"location":e.location,"message":e.message})).collect::<Vec<_>>()})
}
