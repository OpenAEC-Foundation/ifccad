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
    mut diagnostics: Vec<Value>,
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
            crate::ocdraw::record_target_codec_failure(&mut output, &error.to_string());
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
            crate::ocdraw::record_target_codec_failure(&mut output, &error);
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
    diagnostics.extend(spline_parameterization_losses(&document, &checked));
    diagnostics.extend(viewport_shadeplot_losses(&document, &checked));
    output["export"]["diagnostics"] = json!(&diagnostics);
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

// Compare independent outgoing typed parameters with physical reader output.
// Scope name and spline order locate the curve without comparing CAD handles.
fn spline_parameterization_losses(expected: &CadDocument, actual: &CadDocument) -> Vec<Value> {
    use ocdraw_convert::opencadcodec::EntityType;
    let mut losses = Vec::new();
    for block in expected.block_records.iter() {
        let Some(returned) = actual.block_records.get(&block.name) else {
            continue;
        };
        let expected_splines =
            block
                .entity_handles
                .iter()
                .filter_map(|h| match expected.get_entity(*h) {
                    Some(EntityType::Spline(s)) => Some(s),
                    _ => None,
                });
        let actual_splines: Vec<_> = returned
            .entity_handles
            .iter()
            .filter_map(|h| match actual.get_entity(*h) {
                Some(EntityType::Spline(s)) => Some(s),
                _ => None,
            })
            .collect();
        for (index, source) in expected_splines.enumerate() {
            let Some(target) = actual_splines.get(index) else {
                continue;
            };
            if source.control_points.is_empty()
                && !source.fit_points.is_empty()
                && matches!(source.knot_parameterization, 1 | 2)
                && target.knot_parameterization == 0
            {
                losses.push(json!({"code":"TARGET_CODEC_SPLINE_PARAMETERIZATION_LOSS",
                    "location":format!("block/{}/splines/{index}",block.name),"phase":"cadExchange","action":"Modified",
                    "expected":source.knot_parameterization,"actual":target.knot_parameterization,
                    "message":"The target CAD codec changed spline fit parameterization; the resulting curve may differ. Typed source restoration succeeded before this physical exchange loss."}));
            }
        }
    }
    losses
}

// Match frame occurrences within the owning block, allowing a generated overall canvas.
// Handles and runtime viewport numbers are not stable exchange identities.
fn viewport_shadeplot_losses(expected: &CadDocument, actual: &CadDocument) -> Vec<Value> {
    use ocdraw_convert::opencadcodec::EntityType;
    let mut losses = Vec::new();
    for block in expected.block_records.iter() {
        let returned: Vec<_> = actual
            .block_records
            .get(&block.name)
            .into_iter()
            .flat_map(|b| &b.entity_handles)
            .filter_map(|h| match actual.get_entity(*h) {
                Some(EntityType::Viewport(v)) => Some(v),
                _ => None,
            })
            .collect();
        let mut used = vec![false; returned.len()];
        for (index, source) in block
            .entity_handles
            .iter()
            .filter_map(|h| match expected.get_entity(*h) {
                Some(EntityType::Viewport(v)) => Some(v),
                _ => None,
            })
            .enumerate()
        {
            let target = returned
                .iter()
                .enumerate()
                .find(|(i, v)| {
                    !used[*i]
                        && v.center == source.center
                        && v.width == source.width
                        && v.height == source.height
                })
                .map(|(i, v)| {
                    used[i] = true;
                    *v
                });
            let actual_mode = target.map(|v| v.shade_plot_mode);
            if source.shade_plot_mode != 0 && actual_mode != Some(source.shade_plot_mode) {
                losses.push(json!({"code":"TARGET_CODEC_VIEWPORT_SHADEPLOT_LOSS","location":format!("block/{}/viewports/{index}.shadePlot",block.name),"phase":"cadExchange","action":"Modified","expected":source.shade_plot_mode,"actual":actual_mode,"message":"The requested viewport ShadePlot mode was not retained or could not be confirmed in physical CAD readback. The pinned DXF codec lacks VIEWPORT group 170 support; memory/DWG support is qualified separately."}));
            }
        }
    }
    losses
}
