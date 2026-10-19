use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn conversion_capabilities() -> String {
    viewer::conversion_capabilities().to_string()
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
mod tests {
    #[test]
    fn ifccad_hatch_routes_keep_state_and_report_qualified_readback() {
        let input = include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-gradient.ifcx");
        let opened = super::open_ifccad("gradient.ifcx", input);
        let value: serde_json::Value = serde_json::from_str(&opened).unwrap();
        assert!(value["failure"].is_null());
        assert_eq!(value["presentation"]["hatchEntityCount"], 2);
        assert!(opened.contains("partial"));
        let output: serde_json::Value = serde_json::from_str(&super::export_ifccad_with_options(
            "gradient.ifcx",
            input,
            "dxf",
            "AC1032",
            "{}",
        ))
        .unwrap();
        assert!(output["failure"].is_null());
        assert!(output["export"]["download"]["base64"].is_string());
    }
    #[test]
    fn hatch_solid_open_and_gradient_reject_keep_ifccad_evidence() {
        let solid: serde_json::Value = serde_json::from_str(&super::open_ifccad(
            "hatch.ifcx",
            include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-solid.ifcx"),
        ))
        .unwrap();
        assert_eq!(solid["presentation"]["hatchEntityCount"], 1);
        let rejected: serde_json::Value = serde_json::from_str(&super::export_ifccad_with_options(
            "gradient.ifcx",
            include_bytes!("../../../tests/fixtures/ifccad/hello-hatch-gradient.ifcx"),
            "dxf",
            "AC1032",
            r#"{"lossPolicy":"reject"}"#,
        ))
        .unwrap();
        assert_eq!(rejected["failure"]["code"], "CAD_EXCHANGE_LOSS_REJECTED");
        assert!(rejected["export"]["download"].is_null());
    }
    #[test]
    fn parked_input_does_not_parse_as_ifccad() {
        let value: serde_json::Value =
            serde_json::from_str(&super::open_ifccad("old.ocdraw.json", b"{}")).unwrap();
        assert_eq!(value["failure"]["code"], "OCDRAW_PARKED");
        let capabilities: serde_json::Value =
            serde_json::from_str(&super::conversion_capabilities()).unwrap();
        assert!(capabilities.get("ocdraw").is_none());
        assert_eq!(capabilities["ifccad"]["splinePreservation"], true);
    }
}
