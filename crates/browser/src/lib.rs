use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn conversion_capabilities() -> String {
    viewer::conversion_capabilities().to_string()
}

#[wasm_bindgen]
pub fn convert_cad_to_drawing_with_options(
    name: &str,
    format: &str,
    bytes: &[u8],
    options: &str,
) -> String {
    viewer::inspect_cad_as_drawing_bytes_with_options(name, format, bytes, options).to_string()
}
#[wasm_bindgen]
pub fn export_drawing_with_options(
    name: &str,
    bytes: &[u8],
    format: &str,
    version: &str,
    options: &str,
) -> String {
    viewer::export_drawing_bytes_with_options(name, bytes, format, version, options).to_string()
}
#[wasm_bindgen]
pub fn convert_cad_to_ifccad_with_options(
    name: &str,
    format: &str,
    bytes: &[u8],
    timestamp: &str,
    options: &str,
) -> String {
    viewer::inspect_cad_as_ifccad_bytes_with_options(name, format, bytes, timestamp, options)
        .to_string()
}
#[wasm_bindgen]
pub fn export_ifccad_with_options(
    name: &str,
    bytes: &[u8],
    format: &str,
    version: &str,
    options: &str,
) -> String {
    viewer::export_ifccad_bytes_with_options(name, bytes, format, version, options).to_string()
}

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
#[cfg(test)]
mod hatch_tests {
    #[test]
    fn existing_browser_open_routes_expose_hatch_inspection() {
        assert!(super::open_drawing(
            "hatch.ocdraw.json",
            include_bytes!("../../../examples/ocdraw/hello-hatch-solid.ocdraw.json")
        )
        .contains("hatchEntityCount\":1"));
        assert!(super::open_ifccad(
            "hatch.ifcx",
            include_bytes!("../../../examples/ifccad/hello-hatch-solid.ifcx")
        )
        .contains("hatchEntityCount\":1"));
    }
}
