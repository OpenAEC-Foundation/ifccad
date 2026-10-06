use crate::{fail, progress, result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw::ocdraw::{
    load_ocdraw_bytes, load_ocdraw_file, OcdrawOpenError, OcdrawReadError, OcdrawReadStatus,
    ValidatedOcdraw,
};
use ocdraw_convert::opencadcodec::{self, DwgReader, DxfReader};
use ocdraw_convert::{
    cad_document_to_ocdraw_document, ocdraw_source_to_cad_document, CadToOcdrawOptions,
    OcdrawToCadOptions,
};
use serde_json::{json, Value};
use std::io::Cursor;
use std::path::Path;

pub fn inspect_drawing(path: &Path) -> Value {
    if crate::ifccad::is_ifccad_name(&path.to_string_lossy()) {
        return match std::fs::read(path) {
            Ok(bytes) => crate::ifccad::inspect_ifccad_bytes(&path.to_string_lossy(), &bytes),
            Err(error) => {
                let mut output = result(path, "ifccad");
                fail(&mut output, "reading", "IFCCAD_OPEN_FAILED", error);
                output
            }
        };
    }
    let mut output = result(path, "ocdraw");
    match load_ocdraw_file(path) {
        Ok(drawing) => present(&mut output, Ok(drawing)),
        Err(OcdrawOpenError::Read(error)) => present(&mut output, Err(error)),
        Err(error) => fail(&mut output, "reading", "DRAWING_OPEN_FAILED", error),
    }
    output
}

pub fn inspect_drawing_bytes(name: &str, bytes: &[u8]) -> Value {
    if crate::ifccad::is_ifccad_name(name) {
        return crate::ifccad::inspect_ifccad_bytes(name, bytes);
    }
    let mut output = result(Path::new(name), "ocdraw");
    present(&mut output, load_ocdraw_bytes(bytes));
    output
}

pub fn inspect_cad_as_drawing_bytes(name: &str, format: &str, bytes: &[u8]) -> Value {
    inspect_cad_as_drawing_bytes_with_preservation(name, format, bytes, false)
}

pub fn inspect_cad_as_drawing_bytes_with_preservation(
    name: &str,
    format: &str,
    bytes: &[u8],
    capture: bool,
) -> Value {
    let options = crate::options::ConversionOptions {
        preserve_splines: capture,
        ..Default::default()
    };
    inspect_cad_as_drawing_with_options(name, format, bytes, &options)
}
pub fn inspect_cad_as_drawing_bytes_with_options(
    name: &str,
    format: &str,
    bytes: &[u8],
    options: &str,
) -> Value {
    match crate::options::ConversionOptions::parse(options) {
        Ok(o) => inspect_cad_as_drawing_with_options(name, format, bytes, &o),
        Err(e) => crate::options::invalid(name, format, "INVALID_CONVERSION_OPTIONS", e),
    }
}
fn inspect_cad_as_drawing_with_options(
    name: &str,
    format: &str,
    bytes: &[u8],
    options: &crate::options::ConversionOptions,
) -> Value {
    let mut output = result(Path::new(name), format);
    let cad = match format {
        "dxf" => {
            DxfReader::from_reader(Cursor::new(bytes.to_vec())).and_then(|reader| reader.read())
        }
        "dwg" => DwgReader::from_stream(Cursor::new(bytes.to_vec())).read(),
        _ => {
            fail(
                &mut output,
                "reading",
                "CAD_FORMAT_UNSUPPORTED",
                "Choose DXF or DWG",
            );
            return output;
        }
    };
    let cad = match cad {
        Ok(document) => document,
        Err(error) => {
            fail(&mut output, "reading", "CAD_READ_FAILED", error);
            return output;
        }
    };
    output["reader"]["messages"] = json!(cad
        .notifications
        .iter()
        .map(|item| format!("{item:?}"))
        .collect::<Vec<_>>());
    progress("converting");
    let converted = match cad_document_to_ocdraw_document(
        &cad,
        CadToOcdrawOptions {
            geometry_tolerance: options.ocdraw_tolerance().expect("checked options"),
            preservation_capture: if options.preserve_splines {
                ocdraw_convert::OcdrawPreservationCapture::SupportedTyped
            } else {
                ocdraw_convert::OcdrawPreservationCapture::Disabled
            },
            ..Default::default()
        },
    ) {
        Ok(converted) => converted,
        Err(error) => {
            if let Some(report) = error.preservation_report() {
                output["conversion"] = json!({"preservation":preservation_report(report)});
            }
            fail(
                &mut output,
                "converting",
                "DRAWING_CONVERSION_FAILED",
                &error,
            );
            if let ocdraw_convert::CadToOcdrawError::Geometry(ref f) = error {
                output["failure"]["geometry"] = crate::options::geometry_failure(f);
            }
            return output;
        }
    };
    let diagnostics = converted.diagnostics().iter().map(|item| json!({
        "source": format!("{:?}", item.source()),
        "action": format!("{:?}", item.action()),
        "reasons": item.reasons().iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>()
    })).collect::<Vec<_>>();
    output["conversion"] = json!({"diagnostics": diagnostics, "entityCount": converted.entity_mapping().len(),
            "options":options.value(),
            "preservation":preservation_report(converted.preservation_report()),"geometry":geometry_report(converted.geometry_assessment())});
    let mut document = converted.into_document();
    if let Some(preservation) = &mut document.preservation {
        for source in &mut preservation.sources {
            source.origin = if format == "dwg" {
                ocdraw::ocdraw::OcdrawPreservationOrigin::Dwg
            } else {
                ocdraw::ocdraw::OcdrawPreservationOrigin::Dxf
            };
            source.source_version = Some(cad.version.as_str().into());
        }
    }
    let encoded = match ocdraw::ocdraw::encode_ocdraw_document(&document) {
        Ok(encoded) => encoded,
        Err(error) => {
            fail(&mut output, "writing", "DRAWING_ENCODE_FAILED", error);
            return output;
        }
    };
    let drawing_bytes = encoded.bytes();
    progress("validating");
    present(&mut output, load_ocdraw_bytes(drawing_bytes));
    if output["validation"]["strictAvailable"] != true {
        fail(
            &mut output,
            "validating",
            "DRAWING_READBACK_FAILED",
            "converted drawing did not pass strict validation",
        );
        return output;
    }
    if drawing_bytes.len() > 64 * 1024 * 1024 {
        fail(
            &mut output,
            "writing",
            "EXPORT_SIZE_LIMIT",
            "Generated drawing exceeds the 64 MiB download limit",
        );
        return output;
    }
    output["export"] = json!({
        "format": "ocdraw",
        "download": {"format":"ocdraw", "fileName": format!("{}.ocdraw.json", Path::new(name).file_stem().unwrap_or_default().to_string_lossy()),
            "byteLength": drawing_bytes.len(), "base64": STANDARD.encode(drawing_bytes)}
    });
    output
}

pub fn export_drawing_bytes(name: &str, bytes: &[u8], format: &str, version: &str) -> Value {
    export_drawing_with_options(
        name,
        bytes,
        format,
        version,
        &crate::options::ConversionOptions::default(),
    )
}
pub fn export_drawing_bytes_with_options(
    name: &str,
    bytes: &[u8],
    format: &str,
    version: &str,
    options: &str,
) -> Value {
    match crate::options::ConversionOptions::parse(options) {
        Ok(o) => export_drawing_with_options(name, bytes, format, version, &o),
        Err(e) => crate::options::invalid(name, format, "INVALID_CONVERSION_OPTIONS", e),
    }
}
fn export_drawing_with_options(
    name: &str,
    bytes: &[u8],
    format: &str,
    version: &str,
    options: &crate::options::ConversionOptions,
) -> Value {
    if crate::ifccad::is_ifccad_name(name) {
        return crate::export_ifccad_bytes_with_options(
            name,
            bytes,
            format,
            version,
            &options.value().to_string(),
        );
    }
    let mut output = inspect_drawing_bytes(name, bytes);
    if output["validation"]["strictAvailable"] != true {
        fail(
            &mut output,
            "validating",
            "EXPORT_REQUIRES_VALID_DRAWING",
            "Export requires a valid OCDraw drawing",
        );
        return output;
    }
    if format == "ocdraw" {
        output["export"] = json!({"format":"ocdraw","download":{"format":"ocdraw","fileName":format!("{}.ocdraw.json",Path::new(name).file_stem().unwrap_or_default().to_string_lossy()),"byteLength":bytes.len(),"base64":STANDARD.encode(bytes)}});
        return output;
    }
    if let Err((code, message)) = crate::cad::selection(format, version) {
        fail(&mut output, "exporting", code, message);
        return output;
    }
    let read = load_ocdraw_bytes(bytes);
    let drawing = read.as_ref().expect("strict status checked");
    progress("converting");
    let converted = match ocdraw_source_to_cad_document(
        drawing,
        OcdrawToCadOptions {
            geometry_tolerance: options.ocdraw_tolerance().expect("checked options"),
            ..Default::default()
        },
    ) {
        Ok(value) => value,
        Err(error) => {
            if let Some(report) = error.preservation_report() {
                output["conversion"] = json!({"preservation":preservation_report(report)});
            }
            fail(&mut output, "converting", "CAD_CONVERSION_FAILED", &error);
            if let ocdraw_convert::OcdrawToCadError::Geometry(ref f) = error {
                output["failure"]["geometry"] = crate::options::geometry_failure(f);
            }
            return output;
        }
    };
    output["conversion"] = json!({"preservation":preservation_report(converted.preservation_report()),"geometry":geometry_report(converted.geometry_assessment())});
    let mut diagnostics = converted
        .diagnostics()
        .iter()
        .map(|item| {
            json!({
                "code":item.code, "location":item.location, "message":item.message
            })
        })
        .collect::<Vec<_>>();
    if format == "dxf"
        && opencadcodec::entities::ViewportStatusFlags::from_bits(0x10000).to_bits() & 0x10000 == 0
    {
        for viewport in drawing.viewports().iter().filter(|v| v.paper_clip.enabled) {
            diagnostics.push(json!({
                "code":"DXF_VIEWPORT_CLIP_LOSS",
                "location":format!("/entities/{}",viewport.id),
                "message":"The selected DXF codec does not preserve viewport clip activation and boundary references; this export uses the rectangular viewport frame"
            }));
        }
    }
    if format == "dwg" {
        for entity in drawing.geometric_entities() {
            if matches!(
                entity.geometry(),
                ocdraw::ocdraw::DrawingGeometry::SpatialPolyline {
                    line_pattern_generation: ocdraw::ocdraw::LinePatternGeneration::Continuous,
                    ..
                }
            ) {
                diagnostics.push(json!({
                    "code":"DWG_SPATIAL_PATTERN_GENERATION_LOSS",
                    "location":format!("/entities/{}", entity.id()),
                    "message":"The DWG writer restarts the line pattern on each spatial polyline segment; DXF preserves continuous generation"
                }));
            }
        }
    }
    let geometry = crate::options::geometry(converted.geometry_assessment());
    let mut returned = crate::cad::export(
        output,
        converted.into_document(),
        name,
        format,
        version,
        diagnostics,
        |_| Ok(Value::Null),
    );
    returned["export"]["geometry"] = geometry;
    returned["export"]["options"] = options.value();
    returned
}

fn present(output: &mut Value, outcome: Result<ValidatedOcdraw, OcdrawReadError>) {
    let status = match &outcome {
        Ok(_) => "valid",
        Err(error) => match error.status() {
            OcdrawReadStatus::Invalid => "invalid",
            OcdrawReadStatus::UnsupportedVersion => "unsupportedVersion",
            OcdrawReadStatus::Valid => unreachable!("reader errors cannot be valid"),
        },
    };
    let diagnostics = outcome
        .as_ref()
        .err()
        .map(|e| e.diagnostics())
        .unwrap_or_default()
        .iter()
        .map(|item| {
            json!({
                "code": item.code, "location": item.location, "message": item.message
            })
        })
        .collect::<Vec<_>>();
    output["validation"] = json!({
        "strictAvailable": outcome.as_ref().ok().is_some(),
        "status": status,
        "diagnostics": diagnostics
    });
    if let Ok(drawing) = outcome.as_ref() {
        let value = drawing.as_value();
        output["presentation"] = json!({
            "format":"ocdraw",
            "entities":crate::inspection::entities(drawing),
            "drawingId": drawing.drawing_id(),
            "unit": value["header"]["unit"],
            "plotStyleMode": drawing.plot_style_mode(),
            "linePatterns": drawing.line_patterns().iter().map(|p|json!({"id":p.id,"name":p.name,"description":p.description,"pattern":p.pattern})).collect::<Vec<_>>(),
            "linePatternScale": drawing.line_pattern_scale(),
            "layers": value["layers"].as_array().cloned().unwrap_or_default(),
            "layouts": value["layouts"],
            "scopes": value["scopes"],
            "streams":value["streams"],
            "blockDefinitions":value["blockDefinitions"],
            "ucsDefinitions":value["ucsDefinitions"],
            "viewState":value["drawingViewState"],
            "pointDisplay":value["pointDisplay"],
            "modelWindows":value["modelWindows"],
            "paperCanvases":value["paperCanvases"],
            "drawingWorkspaceState": value["drawingWorkspaceState"],
            "opaqueEntityCount":drawing.opaque_entities().len(),
            "opaqueEntities":drawing.opaque_entities().iter().map(|e|json!({"id":e.id.to_string(),"preservationRecordId":e.preservation_record_id.0.to_string(),"visible":e.visible,"nativeLayerId":e.layer_id,"nativeAppearanceAvailable":e.appearance.is_some()})).collect::<Vec<_>>(),
            "preservation":drawing.preservation().map(|p|json!({"version":p.version,"nextRecordId":p.next_record_id.to_string(),
                "sources":p.sources.iter().map(|s|json!({"id":s.id,"provider":s.provider,"providerRevision":s.provider_revision,"origin":match s.origin{ocdraw::ocdraw::OcdrawPreservationOrigin::CadDocument=>"cadDocument",ocdraw::ocdraw::OcdrawPreservationOrigin::Dwg=>"dwg",ocdraw::ocdraw::OcdrawPreservationOrigin::Dxf=>"dxf"},"sourceVersion":s.source_version})).collect::<Vec<_>>(),
                "records":p.records.iter().map(|r|json!({"id":r.id.0.to_string(),"sourceId":r.source_id,"sourceKey":r.source_key,"schema":r.payload.schema,"version":r.payload.version,"byteLength":r.payload.bytes.len(),"category":format!("{:?}",r.category),"role":format!("{:?}",r.role),"dependencyCoverage":format!("{:?}",r.dependency_coverage)})).collect::<Vec<_>>()})),
            "boundsCompleteness":drawing.scopes().iter().map(|s|json!({"scopeId":s.id,"status":if s.entities.is_empty(){"empty"}else if s.bounds.is_none(){"unavailable"}else{"complete"}})).collect::<Vec<_>>(),
            "viewportWorkspaces":value["viewportWorkspaces"],
        });
    }
}

/// Convert CAD to a validated standalone drawing, then export that drawing.
pub fn export_cad_bytes(
    name: &str,
    format: &str,
    bytes: &[u8],
    target: &str,
    version: &str,
) -> Value {
    export_cad_bytes_with_preservation(name, format, bytes, target, version, false)
}

pub fn export_cad_bytes_with_preservation(
    name: &str,
    format: &str,
    bytes: &[u8],
    target: &str,
    version: &str,
    capture: bool,
) -> Value {
    let opening = inspect_cad_as_drawing_bytes_with_preservation(name, format, bytes, capture);
    if opening["failure"].is_object()
        || opening["validation"]["strictAvailable"] != true
        || target == "ocdraw"
    {
        return opening;
    }
    let Some(encoded) = opening["export"]["download"]["base64"].as_str() else {
        return opening;
    };
    let drawing = STANDARD.decode(encoded).expect("internal base64 output");
    let mut output = export_drawing_bytes(name, &drawing, target, version);
    output["source"] = opening["source"].clone();
    output["reader"] = opening["reader"].clone();
    let restoration = output["conversion"].clone();
    output["conversion"] = opening["conversion"].clone();
    if output["conversion"].is_object() {
        output["conversion"]["restoration"] = restoration;
    }
    output
}

fn preservation_report(report: &ocdraw_convert::OcdrawPreservationReport) -> Value {
    json!({"entries":report.entries().iter().map(|e|json!({"recordId":e.record_id.map(|id|id.0.to_string()),"entityId":e.entity_id.map(|id|id.to_string()),
        "sourceId":e.source_id,"sourceKey":e.source_key,"schema":e.schema,"version":e.version,"phase":e.phase,"result":e.result,"reason":e.reason,"location":e.location,"message":e.message})).collect::<Vec<_>>()})
}
fn geometry_report(report: &ocdraw_convert::OcdrawGeometryAssessment) -> Value {
    json!({"complete":report.is_complete(),"assessedNativeStatus":format!("{:?}",report.status()),"assessedNativeEntities":report.assessed_entities(),
        "unassessedSources":report.unassessed_sources().iter().map(|s|format!("{s:?}")).collect::<Vec<_>>()})
}

pub(crate) fn record_target_codec_failure(output: &mut Value, message: &str) {
    if let Some(entries) = output["conversion"]["preservation"]["entries"].as_array_mut() {
        let failures = entries
            .iter()
            .filter(|e| e["result"] == "restoredTyped")
            .map(|e| {
                let mut e = e.clone();
                e["phase"] = json!("cadExchange");
                e["result"] = json!("notRestored");
                e["reason"] = json!("targetCodecRejected");
                e["location"] = json!("/export");
                e["message"] = json!(format!("physical CAD output failed: {message}"));
                e
            })
            .collect::<Vec<_>>();
        entries.extend(failures);
    }
}
