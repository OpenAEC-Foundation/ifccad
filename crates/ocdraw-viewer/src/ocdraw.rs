use crate::{fail, progress, result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw::ocdraw::{
    load_ocdraw_bytes, load_ocdraw_file, OcdrawOpenError, OcdrawReadError, OcdrawReadStatus,
    ValidatedOcdraw,
};
use ocdraw_convert::opencadcodec::{DwgReader, DxfReader};
use ocdraw_convert::{
    cad_document_to_encoded_ocdraw, ocdraw_source_to_cad_document, CadToOcdrawOptions,
    OcdrawToCadOptions,
};
use serde_json::{json, Value};
use std::io::Cursor;
use std::path::Path;

pub fn inspect_drawing(path: &Path) -> Value {
    if crate::ifcx::is_ifcx_name(&path.to_string_lossy()) {
        return match std::fs::read(path) {
            Ok(bytes) => crate::ifcx::inspect_ifcx_bytes(&path.to_string_lossy(), &bytes),
            Err(error) => {
                let mut output = result(path, "ifcx");
                fail(&mut output, "reading", "IFCX_OPEN_FAILED", error);
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
    if crate::ifcx::is_ifcx_name(name) {
        return crate::ifcx::inspect_ifcx_bytes(name, bytes);
    }
    let mut output = result(Path::new(name), "ocdraw");
    present(&mut output, load_ocdraw_bytes(bytes));
    output
}

pub fn inspect_cad_as_drawing_bytes(name: &str, format: &str, bytes: &[u8]) -> Value {
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
    let converted = match cad_document_to_encoded_ocdraw(&cad, CadToOcdrawOptions::default()) {
        Ok(converted) => converted,
        Err(error) => {
            fail(
                &mut output,
                "converting",
                "DRAWING_CONVERSION_FAILED",
                error,
            );
            return output;
        }
    };
    let diagnostics = converted.diagnostics().iter().map(|item| json!({
        "source": format!("{:?}", item.source()),
        "action": format!("{:?}", item.action()),
        "reasons": item.reasons().iter().map(|reason| format!("{reason:?}")).collect::<Vec<_>>()
    })).collect::<Vec<_>>();
    output["conversion"] =
        json!({"diagnostics": diagnostics, "entityCount": converted.entity_mapping().len()});
    let drawing_bytes = converted.encoded().bytes();
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
    if crate::ifcx::is_ifcx_name(name) {
        return crate::ifcx::export_ifcx_bytes(name, bytes, format, version);
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
    let converted = match ocdraw_source_to_cad_document(drawing, OcdrawToCadOptions::default()) {
        Ok(value) => value,
        Err(error) => {
            fail(&mut output, "converting", "CAD_CONVERSION_FAILED", error);
            return output;
        }
    };
    let mut diagnostics = converted
        .diagnostics()
        .iter()
        .map(|item| {
            json!({
                "code":item.code, "location":item.location, "message":item.message
            })
        })
        .collect::<Vec<_>>();
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
    crate::cad::export(
        output,
        converted.into_document(),
        name,
        format,
        version,
        diagnostics,
        |_| Ok(Value::Null),
    )
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
            "viewState":value["viewState"],
            "modelWindows":value["modelWindows"],
            "paperCanvases":value["paperCanvases"],
            "drawingWorkspaceState": value["drawingWorkspaceState"],
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
    let opening = inspect_cad_as_drawing_bytes(name, format, bytes);
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
    output["conversion"] = opening["conversion"].clone();
    output
}
