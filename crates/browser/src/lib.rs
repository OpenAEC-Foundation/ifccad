use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn open_drawing(name: &str, bytes: &[u8]) -> String {
    viewer::inspect_drawing_bytes(name, bytes).to_string()
}

#[wasm_bindgen]
pub fn convert_cad_to_drawing(name: &str, format: &str, bytes: &[u8]) -> String {
    viewer::inspect_cad_as_drawing_bytes(name, format, bytes).to_string()
}

#[wasm_bindgen]
pub fn convert_cad_to_drawing_with_preservation(
    name: &str,
    format: &str,
    bytes: &[u8],
    capture: bool,
) -> String {
    viewer::inspect_cad_as_drawing_bytes_with_preservation(name, format, bytes, capture).to_string()
}

#[wasm_bindgen]
pub fn export_drawing(name: &str, bytes: &[u8], format: &str, version: &str) -> String {
    viewer::export_drawing_bytes(name, bytes, format, version).to_string()
}

#[wasm_bindgen]
pub fn open_ifccad(name: &str, bytes: &[u8]) -> String {
    viewer::inspect_ifccad_bytes(name, bytes).to_string()
}

#[wasm_bindgen]
pub fn convert_cad_to_ifccad(name: &str, format: &str, bytes: &[u8], timestamp: &str) -> String {
    viewer::inspect_cad_as_ifccad_bytes(name, format, bytes, timestamp).to_string()
}

#[wasm_bindgen]
pub fn export_ifccad(name: &str, bytes: &[u8], format: &str, version: &str) -> String {
    viewer::export_ifccad_bytes(name, bytes, format, version).to_string()
}
