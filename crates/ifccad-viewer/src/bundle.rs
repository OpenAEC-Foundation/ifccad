use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::{json, Value};
use std::{
    collections::BTreeSet,
    fs,
    io::Read,
    path::{Path, PathBuf},
};
type Error = Box<dyn std::error::Error>;
fn safe(root: &Path, uri: &str) -> Result<PathBuf, Error> {
    if uri.is_empty()
        || uri.contains(['\\', ':', '\0', '%'])
        || uri.starts_with('/')
        || uri
            .split('/')
            .any(|s| s.is_empty() || s == "." || s == ".." || s.ends_with(['.', ' ']))
    {
        return Err("Unsafe package-relative preview path".into());
    }
    let base = root.canonicalize()?;
    let path = base.join(uri).canonicalize()?;
    if !path.starts_with(&base) {
        return Err("Preview escapes package".into());
    }
    Ok(path)
}
pub fn collect(root: &Path) -> Result<Value, Error> {
    let text = fs::read_to_string(root.join("package.ifcx.json"))?;
    let ifcx: Value = serde_json::from_str(&text)?;
    let mut documents = vec![json!({"path":"package.ifcx.json","text":text})];
    let mut blobs = vec![];
    let mut warnings = vec![];
    let mut seen = BTreeSet::new();
    let mut seen_blobs = BTreeSet::new();
    if let Some(nodes) = ifcx["data"].as_array() {
        for node in nodes {
            for key in ["resource", "preservation"] {
                let expected = if key == "resource" {
                    "openaec:DrawingRepresentation"
                } else {
                    "openaec:PreservationRepresentation"
                };
                if node["type"].as_str() != Some(expected) {
                    continue;
                }
                let d = &node["attributes"][key];
                if d.is_null() {
                    continue;
                }
                let body = if let Some(uri) = d["uri"].as_str() {
                    let text = fs::read_to_string(safe(root, uri)?)?;
                    let body: Value = serde_json::from_str(&text)?;
                    if seen.insert(uri.to_owned()) {
                        documents.push(json!({"path":uri,"text":text}));
                    }
                    body
                } else {
                    d["content"].clone()
                };
                if let Some(items) = body["blobs"].as_array() {
                    for blob in items {
                        if let Some(uri) = blob["uri"].as_str() {
                            if !seen_blobs.insert(uri.to_owned()) {
                                continue;
                            }
                            let preview = (|| -> Result<Value, Error> {
                                let file = fs::File::open(safe(root, uri)?)?;
                                let len = file.metadata()?.len();
                                let mut bytes = vec![];
                                file.take(4096).read_to_end(&mut bytes)?;
                                Ok(
                                    json!({"path":uri,"byteLength":len,"previewBase64":STANDARD.encode(bytes)}),
                                )
                            })();
                            match preview{Ok(b)=>blobs.push(b),Err(_)=>warnings.push(json!({"code":"BLOB_PREVIEW_UNAVAILABLE","resourceId":d["resourceId"],"message":format!("Preview unavailable: {uri}")}))}
                        }
                    }
                }
            }
        }
    }
    Ok(
        json!({"name":root.file_name().unwrap_or_default().to_string_lossy(),"documents":documents,"blobs":blobs,"warnings":warnings}),
    )
}
