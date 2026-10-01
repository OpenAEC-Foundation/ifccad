mod cad;
mod ifcx;
mod ocdraw;
pub use ifcx::{export_ifcx_bytes, inspect_cad_as_ifcx_bytes, inspect_ifcx_bytes};
pub use ocdraw::{
    export_cad_bytes, export_drawing_bytes, inspect_cad_as_drawing_bytes, inspect_drawing,
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
