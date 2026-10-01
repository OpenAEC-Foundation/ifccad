use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn open_drawing(name: &str, bytes: &[u8]) -> String {
    ocdraw_viewer::inspect_drawing_bytes(name, bytes).to_string()
}

#[wasm_bindgen]
pub fn convert_cad_to_drawing(name: &str, format: &str, bytes: &[u8]) -> String {
    ocdraw_viewer::inspect_cad_as_drawing_bytes(name, format, bytes).to_string()
}

#[wasm_bindgen]
pub fn export_drawing(name: &str, bytes: &[u8], format: &str, version: &str) -> String {
    ocdraw_viewer::export_drawing_bytes(name, bytes, format, version).to_string()
}
