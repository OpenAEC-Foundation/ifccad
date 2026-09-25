//! Application export: validated IFCCAD drawing -> CAD document -> downloadable file.
use crate::{fail, inspect_cad, progress, result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad::package::load_directory_package;
use ifccad_convert::{
    cadcodec::{DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter},
    drawing_to_cad_document, ImportDiagnostic,
};
use serde_json::{json, Value};
use std::{io::Cursor, path::Path};

/// Export one drawing from a freshly validated package, retaining scoped diagnostics.
pub fn export_package(root: &Path, drawing_path: &str, format: &str) -> Value {
    export_package_versioned(root, drawing_path, format, "AC1032")
}

/// Export with an explicit CAD version. The package ZIP format has no CAD version.
pub fn export_package_versioned(
    root: &Path,
    drawing_path: &str,
    format: &str,
    version: &str,
) -> Value {
    let mut r = result(root, "package");
    r["export"] = Value::Null;
    if !matches!(format, "dxf" | "dwg" | "ifccad") {
        fail(
            &mut r,
            "exporting",
            "INVALID_EXPORT_FORMAT",
            "Choose DXF, DWG or IFCCAD",
        );
        return r;
    }
    let target_version = if format == "ifccad" {
        None
    } else {
        match version {
            "AC1015" => Some(DxfVersion::AC1015),
            "AC1018" => Some(DxfVersion::AC1018),
            "AC1021" => Some(DxfVersion::AC1021),
            "AC1024" => Some(DxfVersion::AC1024),
            "AC1027" => Some(DxfVersion::AC1027),
            "AC1032" => Some(DxfVersion::AC1032),
            _ => {
                fail(
                    &mut r,
                    "exporting",
                    "INVALID_CAD_VERSION",
                    "Choose a supported CAD version",
                );
                return r;
            }
        }
    };
    progress("validating");
    let loaded = match load_directory_package(root) {
        Ok(loaded) => loaded,
        Err(e) => {
            fail(&mut r, "reading", "PACKAGE_OPEN_FAILED", e);
            return r;
        }
    };
    r["validation"] =
        json!({"strictAvailable":loaded.validated_package().is_some(),"report":loaded.report()});
    let Some(package) = loaded.validated_package() else {
        fail(
            &mut r,
            "validating",
            "EXPORT_REQUIRES_VALID_PACKAGE",
            "Export requires a strictly readable IFCCAD package",
        );
        return r;
    };
    if format == "ifccad" {
        r["export"] = json!({"format":"ifccad","packageReady":true,"drawingCount":package.drawings().count(),"download":null});
        return r;
    }
    let Some(drawing) = package.drawings().find(|d| d.path() == drawing_path) else {
        fail(
            &mut r,
            "exporting",
            "DRAWING_NOT_FOUND",
            "The selected drawing is not in this package",
        );
        return r;
    };
    progress("exporting");
    let converted = match drawing_to_cad_document(drawing) {
        Ok(converted) => converted,
        Err(e) => {
            fail(
                &mut r,
                "exporting",
                "CAD_CONVERSION_FAILED",
                format!("{e:?}"),
            );
            return r;
        }
    };
    let assessment = converted.transfer_assessment();
    let geometry = converted.geometry_assessment();
    let diagnostics: Vec<_> = converted
        .diagnostics()
        .iter()
        .map(|d| {
            let code = match d {
                ImportDiagnostic::GeometryRoundedWithinTolerance { .. } => {
                    "GeometryRoundedWithinTolerance"
                }
                ImportDiagnostic::PlaneParameterizationChanged { .. } => {
                    "PlaneParameterizationChanged"
                }
                ImportDiagnostic::LinePatternFallback { .. } => "LinePatternFallback",
                ImportDiagnostic::LineWeightRounded { .. } => "LineWeightRounded",
                _ => "ImportDiagnostic",
            };
            json!({"code":code,"message":d.to_string(),"details":format!("{d:?}")})
        })
        .collect();
    r["export"] = json!({
        "drawing":drawing_path,"resourceId":drawing.representation().resource_id().as_str(),"format":format,
        "requestedVersion":version,"effectiveVersion":null,
        "assessment":{"conclusion":format!("{:?}",assessment.conclusion()),"coverage":format!("{:?}",assessment.coverage()),"scope":format!("{:?}",assessment.scope()),"limitations":assessment.limitations()},
        "geometry":{"status":format!("{:?}",geometry.status()),"drawingUnit":format!("{:?}",geometry.drawing_unit()),"requestedTolerance":format!("{:?}",geometry.requested_tolerance()),"maxDeviationUpperBound":geometry.max_deviation_upper_bound(),"assessedEntities":geometry.assessed_entities(),"assessedVertices":geometry.assessed_vertices()},
        "diagnostics":diagnostics,"entityCount":converted.entity_mapping().len(),
        "preservationRestored":false,"fileCheck":null,"download":null
    });
    progress("writing");
    let mut document = converted.document().clone();
    document.version = target_version.expect("CAD version checked above");
    let written = if format == "dxf" {
        DxfWriter::new(&document).write_to_vec()
    } else {
        DwgWriter::write_to_vec(&document)
    };
    let bytes = match written {
        Ok(bytes) => bytes,
        Err(e) => {
            fail(&mut r, "writing", "CAD_WRITE_FAILED", e);
            return r;
        }
    };
    if bytes.len() > 64 * 1024 * 1024 {
        fail(
            &mut r,
            "writing",
            "EXPORT_SIZE_LIMIT",
            "Generated CAD file exceeds the 64 MiB download limit",
        );
        return r;
    }
    progress("checking");
    // Readability is separate from semantic fidelity and the converter's geometry proof.
    let readback = if format == "dxf" {
        DxfReader::from_reader(Cursor::new(bytes.clone())).and_then(|reader| reader.read())
    } else {
        DwgReader::from_stream(Cursor::new(bytes.clone())).read()
    };
    match readback {
        Ok(doc) => {
            if doc.version != document.version {
                fail(
                    &mut r,
                    "checking",
                    "CAD_VERSION_MISMATCH",
                    format!("Requested {version}, written {}", doc.version.as_str()),
                );
                return r;
            }
            r["export"]["effectiveVersion"] = json!(doc.version.as_str());
            r["export"]["fileCheck"] = json!({"readable":true,"entityCount":doc.entities().count(),"messages":doc.notifications.iter().map(|n|format!("{n:?}")).collect::<Vec<_>>(),"semanticFidelityAssessed":false})
        }
        Err(e) => {
            fail(&mut r, "checking", "CAD_READBACK_FAILED", e);
            return r;
        }
    }
    r["export"]["download"] =
        json!({"format":format,"byteLength":bytes.len(),"base64":STANDARD.encode(bytes)});
    r
}

/// Recreate the native package from the unchanged selected CAD input before export.
/// Original CAD bytes are never offered as if they were an IFCCAD export.
pub fn export_cad(input: &Path, output: &Path, drawing: &str, format: &str) -> Value {
    export_cad_versioned(input, output, drawing, format, "AC1032")
}

/// Recreate and export one drawing at the requested CAD version.
pub fn export_cad_versioned(
    input: &Path,
    output: &Path,
    drawing: &str,
    format: &str,
    version: &str,
) -> Value {
    let opening = inspect_cad(input, output);
    if !opening["failure"].is_null() || opening["validation"]["strictAvailable"] != true {
        return opening;
    }
    let mut exported = export_package_versioned(output, drawing, format, version);
    exported["source"] = opening["source"].clone();
    exported["conversion"] = opening["conversion"].clone();
    exported["reader"] = opening["reader"].clone();
    exported
}
