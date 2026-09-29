use crate::{fail, progress, result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw::drawing::{
    load_drawing_bytes, load_drawing_file, DrawingLoadOutcome, DrawingLoadStatus,
};
use ocdraw_convert::cadcodec::{DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter};
use ocdraw_convert::{
    cad_document_to_drawing, ocdraw_to_cad_document, ExportOptions, ImportOptions,
};
use serde_json::{json, Value};
use std::io::Cursor;
use std::path::Path;

pub fn inspect_drawing(path: &Path) -> Value {
    let mut output = result(path, "ocdraw");
    match load_drawing_file(path) {
        Ok(drawing) => present(&mut output, drawing),
        Err(error) => fail(&mut output, "reading", "DRAWING_OPEN_FAILED", error),
    }
    output
}

pub fn inspect_drawing_bytes(name: &str, bytes: &[u8]) -> Value {
    let mut output = result(Path::new(name), "ocdraw");
    present(&mut output, load_drawing_bytes(bytes));
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
    let converted = match cad_document_to_drawing(&cad, ExportOptions::default()) {
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
    let drawing_bytes = converted.drawing().bytes();
    progress("validating");
    present(&mut output, load_drawing_bytes(drawing_bytes));
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
    let cad_version = match version {
        "AC1015" => DxfVersion::AC1015,
        "AC1018" => DxfVersion::AC1018,
        "AC1021" => DxfVersion::AC1021,
        "AC1024" => DxfVersion::AC1024,
        "AC1027" => DxfVersion::AC1027,
        "AC1032" => DxfVersion::AC1032,
        _ => {
            fail(
                &mut output,
                "exporting",
                "INVALID_CAD_VERSION",
                "Choose a supported CAD version",
            );
            return output;
        }
    };
    if !matches!(format, "dxf" | "dwg") {
        fail(
            &mut output,
            "exporting",
            "INVALID_EXPORT_FORMAT",
            "Choose DXF or DWG",
        );
        return output;
    }
    let read = load_drawing_bytes(bytes);
    let drawing = read.validated_drawing().expect("strict status checked");
    progress("converting");
    let converted = match ocdraw_to_cad_document(drawing, ImportOptions::default()) {
        Ok(value) => value,
        Err(error) => {
            fail(&mut output, "converting", "CAD_CONVERSION_FAILED", error);
            return output;
        }
    };
    let diagnostics = converted
        .diagnostics()
        .iter()
        .map(|item| {
            json!({
                "code":item.code, "location":item.location, "message":item.message
            })
        })
        .collect::<Vec<_>>();
    let mut cad = converted.into_document();
    cad.version = cad_version;
    progress("writing");
    let result = if format == "dxf" {
        DxfWriter::new(&cad).write_to_vec()
    } else {
        DwgWriter::write_to_vec(&cad)
    };
    let written = match result {
        Ok(bytes) => bytes,
        Err(error) => {
            fail(&mut output, "writing", "CAD_WRITE_FAILED", error);
            return output;
        }
    };
    if written.len() > 64 * 1024 * 1024 {
        fail(
            &mut output,
            "writing",
            "EXPORT_SIZE_LIMIT",
            "Generated CAD file exceeds the 64 MiB download limit",
        );
        return output;
    }
    progress("checking");
    let checked = if format == "dxf" {
        DxfReader::from_reader(Cursor::new(written.clone())).and_then(|reader| reader.read())
    } else {
        DwgReader::from_stream(Cursor::new(written.clone())).read()
    };
    match checked {
        Ok(readback) if readback.version == cad_version => {}
        Ok(readback) => {
            fail(
                &mut output,
                "checking",
                "CAD_VERSION_MISMATCH",
                format!("requested {version}, written {}", readback.version.as_str()),
            );
            return output;
        }
        Err(error) => {
            fail(&mut output, "checking", "CAD_READBACK_FAILED", error);
            return output;
        }
    }
    output["export"] = json!({
        "format":format, "requestedVersion":version, "diagnostics":diagnostics,
        "download":{"format":format,"fileName":format!("{}.{}", Path::new(name).file_stem().unwrap_or_default().to_string_lossy(), format),
            "byteLength":written.len(),"base64":STANDARD.encode(written)}
    });
    output
}

fn present(output: &mut Value, outcome: DrawingLoadOutcome) {
    let status = match outcome.status() {
        DrawingLoadStatus::Valid => "valid",
        DrawingLoadStatus::Invalid => "invalid",
        DrawingLoadStatus::UnsupportedVersion => "unsupportedVersion",
    };
    let diagnostics = outcome
        .diagnostics()
        .iter()
        .map(|item| {
            json!({
                "code": item.code, "location": item.location, "message": item.message
            })
        })
        .collect::<Vec<_>>();
    output["validation"] = json!({
        "strictAvailable": outcome.validated_drawing().is_some(),
        "status": status,
        "diagnostics": diagnostics
    });
    if let Some(drawing) = outcome.validated_drawing() {
        let value = drawing.as_value();
        output["presentation"] = json!({
            "drawingId": drawing.drawing_id(),
            "unit": value["header"]["unit"],
            "plotStyleMode": drawing.plot_style_mode(),
            "layers": drawing.layers(),
            "layouts": drawing.layouts(),
            "scopes": value["scopes"],
            "streamDirectory": value["streamDirectory"],
            "drawingWorkspaceState": value["drawingWorkspaceState"],
        });
    }
}
