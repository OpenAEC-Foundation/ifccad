//! Shared physical CAD IO; the two drawing converters remain independent.
use crate::{fail, progress};
use base64::{engine::general_purpose::STANDARD, Engine};
use ocdraw_convert::opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter,
};
use serde_json::{json, Value};
use std::{io::Cursor, path::Path};

pub(crate) fn selection(
    format: &str,
    version: &str,
) -> Result<DxfVersion, (&'static str, &'static str)> {
    if !matches!(format, "dxf" | "dwg") {
        return Err(("INVALID_EXPORT_FORMAT", "Choose DXF or DWG"));
    }
    match version {
        "AC1015" => Ok(DxfVersion::AC1015),
        "AC1018" => Ok(DxfVersion::AC1018),
        "AC1021" => Ok(DxfVersion::AC1021),
        "AC1024" => Ok(DxfVersion::AC1024),
        "AC1027" => Ok(DxfVersion::AC1027),
        "AC1032" => Ok(DxfVersion::AC1032),
        _ => Err(("INVALID_CAD_VERSION", "Choose a supported CAD version")),
    }
}
pub(crate) fn read(format: &str, bytes: &[u8]) -> Result<CadDocument, String> {
    match format {
        "dxf" => DxfReader::from_reader(Cursor::new(bytes.to_vec()))
            .and_then(|r| r.read())
            .map_err(|e| e.to_string()),
        "dwg" => DwgReader::from_stream(Cursor::new(bytes.to_vec()))
            .read()
            .map_err(|e| e.to_string()),
        _ => Err("Choose DXF or DWG".into()),
    }
}
pub(crate) fn download(output: &mut Value, name: &str, format: &str, bytes: &[u8]) {
    if bytes.len() > 64 * 1024 * 1024 {
        fail(
            output,
            "writing",
            "EXPORT_SIZE_LIMIT",
            "Generated file exceeds the 64 MiB download limit",
        );
        return;
    }
    let stem = Path::new(name)
        .file_stem()
        .unwrap_or_default()
        .to_string_lossy();
    let extension = if format == "ifccad" { "ifcx" } else { format };
    output["export"] = json!({"format":format,"download":{"format":format,"fileName":format!("{stem}.{extension}"),"byteLength":bytes.len(),"base64":STANDARD.encode(bytes)}});
}

pub(crate) fn export(
    mut output: Value,
    mut document: CadDocument,
    name: &str,
    format: &str,
    version: &str,
    diagnostics: Vec<Value>,
    check: impl FnOnce(&CadDocument) -> Result<Value, String>,
) -> Value {
    output["export"] =
        json!({"format":format,"requestedVersion":version,"diagnostics":diagnostics});
    let cad_version = match selection(format, version) {
        Ok(v) => v,
        Err((code, message)) => {
            fail(&mut output, "exporting", code, message);
            return output;
        }
    };
    document.version = cad_version;
    progress("writing");
    let written = if format == "dxf" {
        DxfWriter::new(&document).write_to_vec()
    } else {
        DwgWriter::write_to_vec(&document)
    };
    let bytes = match written {
        Ok(v) => v,
        Err(error) => {
            fail(&mut output, "writing", "CAD_WRITE_FAILED", error);
            return output;
        }
    };
    if bytes.len() > 64 * 1024 * 1024 {
        fail(
            &mut output,
            "writing",
            "EXPORT_SIZE_LIMIT",
            "Generated CAD file exceeds the 64 MiB download limit",
        );
        return output;
    }
    progress("checking");
    let checked = match read(format, &bytes) {
        Ok(v) => v,
        Err(error) => {
            fail(&mut output, "checking", "CAD_READBACK_FAILED", error);
            return output;
        }
    };
    if checked.version != cad_version {
        fail(
            &mut output,
            "checking",
            "CAD_VERSION_MISMATCH",
            format!("requested {version}, written {}", checked.version.as_str()),
        );
        return output;
    }
    let check = match check(&checked) {
        Ok(v) => v,
        Err(error) => {
            fail(&mut output, "checking", "DRAWING_READBACK_FAILED", error);
            return output;
        }
    };
    download(&mut output, name, format, &bytes);
    if output["failure"].is_null() {
        output["export"]["requestedVersion"] = json!(version);
        output["export"]["diagnostics"] = json!(diagnostics);
        output["export"]["fileCheck"] = check;
    }
    output
}
