use crate::{cad, fail, progress, result};
use ifcx_cad_convert::{
    cad_document_to_ifcx_cad, ifcx_cad_to_cad_document, IfcxCadDiagnostic, IfcxCadTargetMetadata,
};
use ocdraw::ifcx_cad::{read_native_cad_ifcx, IfcxCadHeader, ValidatedIfcxCad};
use serde_json::{json, Value};
use std::path::Path;

pub(crate) fn is_ifcx_name(name: &str) -> bool {
    let name = name.to_ascii_lowercase();
    name.ends_with(".ifcx") || name.ends_with(".ifcx.json")
}
fn diagnostics(items: &[IfcxCadDiagnostic]) -> Vec<Value> {
    items.iter().map(|d|json!({"code":d.code,"location":d.location,"message":d.message,"action":format!("{:?}",d.action)})).collect()
}
fn present(output: &mut Value, drawing: &ValidatedIfcxCad) {
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
    output["validation"] = json!({"strictAvailable":true,"status":"valid","diagnostics":[]});
    output["presentation"] = json!({"format":"ifcx","drawingId":d.drawing_id,"unit":d.length_unit,"linePatternScale":d.line_pattern_scale,
        "layers":role("ifccad::layer"),"layouts":role("ifccad::layout"),"blockDefinitions":role("ifccad::blockDefinition"),
        "linePatterns":role("ifccad::linePattern"),"entities":role("ifccad::entity"),"graph":drawing.graph().composed_ifcx()});
}
/// Inspect the experimental IFCX-CAD profile after production composition/validation.
pub fn inspect_ifcx_bytes(name: &str, bytes: &[u8]) -> Value {
    let mut output = result(Path::new(name), "ifcx");
    match read_native_cad_ifcx(bytes) {
        Ok(drawing) => present(&mut output, &drawing),
        Err(report) => {
            output["validation"] = json!({"strictAvailable":false,"status":"invalid","diagnostics":report.errors.iter().map(|e|json!({"code":"IFCX_PROFILE","message":e})).collect::<Vec<_>>()});
            fail(
                &mut output,
                "validating",
                "IFCX_PROFILE_INVALID",
                report.errors.join("; "),
            );
        }
    }
    output
}
/// Convert original CAD bytes directly to IFCX-CAD; no OCDraw projection is used.
pub fn inspect_cad_as_ifcx_bytes(name: &str, format: &str, bytes: &[u8], timestamp: &str) -> Value {
    let mut output = result(Path::new(name), format);
    let cad = match cad::read(format, bytes) {
        Ok(d) => d,
        Err(error) => {
            fail(&mut output, "reading", "CAD_READ_FAILED", error);
            return output;
        }
    };
    output["reader"]["messages"] = json!(cad
        .notifications
        .iter()
        .map(|i| format!("{i:?}"))
        .collect::<Vec<_>>());
    let metadata = IfcxCadTargetMetadata {
        drawing_id: 1,
        header: IfcxCadHeader {
            id: name.into(),
            data_version: "0.1.0".into(),
            author: "Open CAD Drawing explorer".into(),
            timestamp: timestamp.into(),
        },
    };
    progress("converting");
    let converted = match cad_document_to_ifcx_cad(&cad, metadata) {
        Ok(v) => v,
        Err(error) => {
            fail(&mut output, "converting", "IFCX_CONVERSION_FAILED", error);
            return output;
        }
    };
    output["conversion"] = json!({"format":"ifcx","diagnostics":diagnostics(converted.diagnostics()),"entityCount":converted.mappings().entities.iter().count()});
    present(&mut output, converted.validated_ifcx());
    cad::download(&mut output, name, "ifcx", converted.ifcx_bytes());
    output
}
/// Preserve original IFCX on native download; project supported content for CAD.
pub fn export_ifcx_bytes(name: &str, bytes: &[u8], format: &str, version: &str) -> Value {
    let mut output = inspect_ifcx_bytes(name, bytes);
    if output["validation"]["strictAvailable"] != true {
        return output;
    }
    if format == "ifcx" {
        cad::download(&mut output, name, "ifcx", bytes);
        return output;
    }
    if let Err((code, message)) = cad::selection(format, version) {
        fail(&mut output, "exporting", code, message);
        return output;
    }
    let drawing = read_native_cad_ifcx(bytes).expect("strict status checked");
    progress("converting");
    let converted = match ifcx_cad_to_cad_document(&drawing) {
        Ok(v) => v,
        Err(error) => {
            fail(&mut output, "converting", "CAD_CONVERSION_FAILED", error);
            return output;
        }
    };
    let issues = diagnostics(converted.diagnostics());
    let metadata = IfcxCadTargetMetadata {
        header: drawing.document().header.clone(),
        drawing_id: drawing.document().drawing_id,
    };
    cad::export(
        output,
        converted.into_document(),
        name,
        format,
        version,
        issues,
        move |readback| {
            let restored =
                cad_document_to_ifcx_cad(readback, metadata).map_err(|e| e.to_string())?;
            read_native_cad_ifcx(restored.ifcx_bytes()).map_err(|e| e.errors.join("; "))?;
            Ok(
                json!({"cadReadback":true,"ifcxStrictReadback":true,"diagnostics":diagnostics(restored.diagnostics())}),
            )
        },
    )
}
