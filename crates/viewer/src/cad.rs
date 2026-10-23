//! Shared physical CAD IO; the two drawing converters remain independent.
use crate::{fail, progress};
use base64::{engine::general_purpose::STANDARD, Engine};
use ifccad_convert::opencadcodec::{
    CadDocument, DwgReader, DwgWriter, DxfReader, DxfVersion, DxfWriter,
};
use serde_json::{json, Value};
use std::{io::Cursor, path::Path};

pub(crate) const VERSIONS: [DxfVersion; 6] = [
    DxfVersion::AC1015,
    DxfVersion::AC1018,
    DxfVersion::AC1021,
    DxfVersion::AC1024,
    DxfVersion::AC1027,
    DxfVersion::AC1032,
];

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

pub(crate) struct CadFileExportRequest<'a> {
    pub name: &'a str,
    pub format: &'a str,
    pub version: &'a str,
    pub loss_policy: crate::options::ExportLossPolicy,
}
fn header_matches(format: &str, bytes: &[u8], version: &str) -> bool {
    if format == "dwg" {
        return bytes.get(..6) == Some(version.as_bytes());
    }
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text.lines().map(str::trim);
    while let Some(line) = lines.next() {
        if line == "$ACADVER" {
            return lines.next() == Some("1") && lines.next() == Some(version);
        }
    }
    false
}

pub(crate) fn export(
    mut output: Value,
    mut document: CadDocument,
    request: CadFileExportRequest<'_>,
    mut diagnostics: Vec<Value>,
    check: impl FnOnce(&CadDocument) -> Result<Value, String>,
) -> Value {
    let CadFileExportRequest {
        name,
        format,
        version,
        loss_policy,
    } = request;
    output["export"] =
        json!({"format":format,"requestedVersion":version,"diagnostics":diagnostics});
    let cad_version = match selection(format, version) {
        Ok(v) => v,
        Err((code, message)) => {
            fail(&mut output, "exporting", code, message);
            return output;
        }
    };
    let target = crate::hatch_capabilities::HatchTargetProfile {
        format,
        version: cad_version,
    };
    if let Some(restriction) = target.restrictions(&document).into_iter().find(|r| r.hard) {
        fail(
            &mut output,
            "exporting",
            restriction.code,
            format!(
                "{format} {version}: {} ({})",
                restriction.reason, restriction.location
            ),
        );
        return output;
    }
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
            crate::cad::record_target_codec_failure(&mut output, &error.to_string());
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
            crate::cad::record_target_codec_failure(&mut output, &error);
            fail(&mut output, "checking", "CAD_READBACK_FAILED", error);
            return output;
        }
    };
    if checked.dwg_source_version.unwrap_or(checked.version) != cad_version
        || !header_matches(format, &bytes, version)
    {
        fail(
            &mut output,
            "checking",
            "CAD_VERSION_MISMATCH",
            format!("requested {version}, written {}", checked.version.as_str()),
        );
        return output;
    }
    let original_count = diagnostics.len();
    let hatch_issues =
        match crate::hatch_exchange::check_hatch_exchange_for_target(&document, &checked, target) {
            Ok(v) => v,
            Err(error) => {
                fail(&mut output, "checking", "HATCH_READBACK_UNCONFIRMED", error);
                return output;
            }
        };
    diagnostics.extend(hatch_issues.into_iter().map(|i| i.value()));
    diagnostics.extend(crate::line_pattern_exchange::losses(&document, &checked));
    diagnostics.extend(spline_parameterization_losses(&document, &checked));
    diagnostics.extend(viewport_shadeplot_losses(&document, &checked));
    diagnostics.extend(hatch_description_losses(&document, &checked));
    output["export"]["diagnostics"] = json!(&diagnostics);
    if loss_policy == crate::options::ExportLossPolicy::Reject
        && (diagnostics.len() > original_count
            || diagnostics.iter().any(|d| d["phase"] == "cadExchange"))
    {
        fail(
            &mut output,
            "checking",
            "CAD_EXCHANGE_LOSS_REJECTED",
            "CAD file losses were rejected; no download is offered",
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

// Compare independent outgoing typed parameters with physical reader output.
// Scope name and spline order locate the curve without comparing CAD handles.
fn spline_parameterization_losses(expected: &CadDocument, actual: &CadDocument) -> Vec<Value> {
    use ifccad_convert::opencadcodec::EntityType;
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
    use ifccad_convert::opencadcodec::EntityType;
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
                losses.push(json!({"code":"TARGET_CODEC_VIEWPORT_SHADEPLOT_LOSS","location":format!("block/{}/viewports/{index}.shadePlot",block.name),"phase":"cadExchange","action":"Modified","expected":source.shade_plot_mode,"actual":actual_mode,"message":"The requested viewport ShadePlot mode was not retained or could not be confirmed in physical CAD readback."}));
            }
        }
    }
    losses
}

// Scope and typed Hatch order locate metadata independently of fresh handles.
fn hatch_description_losses(expected: &CadDocument, actual: &CadDocument) -> Vec<Value> {
    use ifccad_convert::opencadcodec::EntityType;
    let mut losses = Vec::new();
    for block in expected.block_records.iter() {
        let returned: Vec<_> = actual
            .block_records
            .get(&block.name)
            .into_iter()
            .flat_map(|b| &b.entity_handles)
            .filter_map(|h| match actual.get_entity(*h) {
                Some(EntityType::Hatch(h)) => Some(h),
                _ => None,
            })
            .collect();
        for (index, h) in block
            .entity_handles
            .iter()
            .filter_map(|h| match expected.get_entity(*h) {
                Some(EntityType::Hatch(h)) => Some(h),
                _ => None,
            })
            .enumerate()
        {
            let observed = returned.get(index).map(|h| h.pattern.description.as_str());
            if !h.pattern.description.is_empty() && observed != Some(h.pattern.description.as_str())
            {
                losses.push(json!({"code":"TARGET_CODEC_HATCH_DESCRIPTION_LOSS",
                    "location":format!("block/{}/hatches/{index}.pattern.description",block.name),
                    "phase":"cadExchange","action":"Modified","expected":h.pattern.description,
                    "actual":observed,"message":"The pattern description was not retained in the CAD file."}));
            }
        }
    }
    losses
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
