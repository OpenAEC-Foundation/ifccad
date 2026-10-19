#![recursion_limit = "256"]
mod binary_download;
mod cad;
mod geometry;
mod hatch_exchange;
mod ifccad;
mod ifccad_input;
mod inspection_dimension_label;
mod inspection_ifccad_dimension;
mod options;
mod selection;
pub use binary_download::export_ifccad_binary_bytes;
pub use ifccad::{export_ifccad_bytes, inspect_cad_as_ifccad_bytes, inspect_ifccad_bytes};
use serde_json::{json, Value};
use std::path::Path;
pub fn conversion_capabilities() -> Value {
    json!({"ifccad":{"adjustableTolerance":true,"coordinateToleranceDefault":true,"coordinateToleranceFallback":true,"splinePreservation":true}})
}
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
pub fn inspect_ifccad(path: &Path) -> Value {
    match std::fs::read(path) {
        Ok(bytes) => inspect_ifccad_bytes(&path.to_string_lossy(), &bytes),
        Err(error) => {
            let mut output = result(path, "ifccad");
            fail(&mut output, "reading", "IFCCAD_OPEN_FAILED", error);
            output
        }
    }
}
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
