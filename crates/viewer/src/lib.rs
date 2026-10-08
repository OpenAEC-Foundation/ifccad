mod cad;
mod geometry;
mod ifccad;
mod inspection;
mod options;
mod selection;
pub fn conversion_capabilities() -> Value {
    json!({"ocdraw":{"adjustableTolerance":true,"splinePreservation":true},"ifccad":{"adjustableTolerance":true,"splinePreservation":true}})
}
pub use ocdraw::{export_drawing_bytes_with_options, inspect_cad_as_drawing_bytes_with_options};

pub fn export_ifccad_bytes_with_options(
    name: &str,
    bytes: &[u8],
    format: &str,
    version: &str,
    options: &str,
) -> Value {
    match options::ConversionOptions::parse(options) {
        Ok(o) => ifccad::export_ifccad_with_options(name, bytes, format, version, &o),
        Err(e) => options::invalid(name, "ifccad", "INVALID_CONVERSION_OPTIONS", e),
    }
}
pub fn inspect_cad_as_ifccad_bytes_with_options(
    name: &str,
    format: &str,
    bytes: &[u8],
    timestamp: &str,
    options: &str,
) -> Value {
    match options::ConversionOptions::parse(options) {
        Ok(o) => ifccad::inspect_cad_as_ifccad_with_options(name, format, bytes, timestamp, &o),
        Err(e) => options::invalid(name, "ifccad", "INVALID_CONVERSION_OPTIONS", e),
    }
}
mod ocdraw;
pub use ifccad::{export_ifccad_bytes, inspect_cad_as_ifccad_bytes, inspect_ifccad_bytes};
pub use ocdraw::{
    export_cad_bytes, export_cad_bytes_with_preservation, export_drawing_bytes,
    inspect_cad_as_drawing_bytes, inspect_cad_as_drawing_bytes_with_preservation, inspect_drawing,
    inspect_drawing_bytes,
};
use serde_json::{json, Value};
use std::path::Path;
pub fn result(path: &Path, format: &str) -> Value {
    json!({"source":{"name":path.file_name().unwrap_or_default().to_string_lossy(),"format":format},"reader":{"status":"ok","messages":[]},"conversion":null,"validation":null,"presentation":null,"failure":null})
}
pub fn fail(result: &mut Value, stage: &str, code: &str, message: impl ToString) {
    result["failure"] = json!({"stage":stage,"code":code,"message":message.to_string()});
    if stage == "reading" {
        result["reader"]["status"] = json!("failed");
    }
}
pub fn progress(phase: &str) {
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"progress","phase":phase})
    );
}
