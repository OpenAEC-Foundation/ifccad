use crate::{cad, fail, inspect_ifccad_bytes};
use ifccad::encode_ifccad_cbor_document;
use serde_json::Value;

/// Download the validated drawing projection using an explicit experimental encoding.
/// IFCX source fragments and foreign nodes are outside this projection.
pub fn export_ifccad_binary_bytes(name: &str, bytes: &[u8], encoding: &str) -> Value {
    let mut output = inspect_ifccad_bytes(name, bytes);
    if output["validation"]["strictAvailable"] != true {
        return output;
    }
    let drawing = crate::ifccad_input::load(name, bytes).expect("strict status checked");
    let encoded = if encoding == crate::ifccad_input::encoding(name)
        && matches!(encoding, "cbor" | "cbor-zstd")
    {
        Ok(bytes.to_vec())
    } else {
        match encoding {
            "cbor" => encode_ifccad_cbor_document(drawing.document())
                .map(|encoded| encoded.into_bytes())
                .map_err(|error| error.to_string()),
            "cbor-zstd" => encode_zstd(drawing.document()),
            _ => Err("Choose cbor or cbor-zstd".into()),
        }
    };
    match encoded {
        Ok(bytes) => {
            let extension = if encoding == "cbor" {
                "ifccad.cbor"
            } else {
                "ifccad.cbor.zst"
            };
            cad::download(&mut output, name, extension, &bytes);
            if output["export"]["download"].is_object() {
                let stem = crate::ifccad_input::stem(name);
                output["export"]["download"]["fileName"] = format!("{stem}.{extension}").into();
            }
        }
        Err(error) => fail(
            &mut output,
            "exporting",
            "IFCCAD_BINARY_EXPORT_FAILED",
            error,
        ),
    }
    output
}

#[cfg(feature = "ifccad-cbor-zstd")]
fn encode_zstd(document: &ifccad::IfccadDocument) -> Result<Vec<u8>, String> {
    ifccad::encode_ifccad_cbor_zstd_document(document, Default::default())
        .map(|encoded| encoded.into_bytes())
        .map_err(|error| error.to_string())
}

#[cfg(not(feature = "ifccad-cbor-zstd"))]
fn encode_zstd(_: &ifccad::IfccadDocument) -> Result<Vec<u8>, String> {
    Err("Zstd download is unavailable in this processor build".into())
}
