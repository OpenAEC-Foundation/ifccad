use js_sys::{Array, Uint8Array};
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn open_cad(name: &str, format: &str, bytes: &[u8], timestamp: &str) -> String {
    ifccad_viewer::inspect_cad_bytes(name, format, bytes, timestamp).to_string()
}

#[wasm_bindgen]
pub fn open_package(name: &str, paths: Array, contents: Array) -> Result<String, JsValue> {
    let files = package_files(paths, contents)?;
    Ok(ifccad_viewer::inspect_package_files(name, &files).to_string())
}

#[wasm_bindgen]
pub fn export_package(
    name: &str,
    paths: Array,
    contents: Array,
    drawing: &str,
    format: &str,
    version: &str,
) -> Result<String, JsValue> {
    let files = package_files(paths, contents)?;
    Ok(ifccad_viewer::export_package_files(name, &files, drawing, format, version).to_string())
}

fn package_files(paths: Array, contents: Array) -> Result<BTreeMap<String, Vec<u8>>, JsValue> {
    if paths.length() != contents.length() {
        return Err(JsValue::from_str("Package path and byte counts differ"));
    }
    let mut files = BTreeMap::new();
    for index in 0..paths.length() {
        let path = paths
            .get(index)
            .as_string()
            .ok_or_else(|| JsValue::from_str("Package path is not a string"))?;
        let bytes = Uint8Array::new(&contents.get(index)).to_vec();
        if files.insert(path, bytes).is_some() {
            return Err(JsValue::from_str("Duplicate package path"));
        }
    }
    Ok(files)
}
