mod bundle;
mod cad;
mod cad_export;
mod package;
pub use cad::inspect_cad;
pub use cad_export::{export_cad, export_package};
pub use package::inspect_package;
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
