//! Bounded downloads of already validated output bytes.
use crate::fail;
use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::path::Path;

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
