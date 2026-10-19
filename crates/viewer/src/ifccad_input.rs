use ifccad::{IfccadDocument, ValidatedIfccad, ValidatedIfccadCbor};

pub(crate) enum IfccadInput {
    Ifcx(ValidatedIfccad),
    Binary(ValidatedIfccadCbor),
}

impl IfccadInput {
    pub(crate) fn document(&self) -> &IfccadDocument {
        match self {
            Self::Ifcx(value) => value.document(),
            Self::Binary(value) => value.document(),
        }
    }

    pub(crate) fn to_cad(
        &self,
        options: ifccad_convert::IfccadToCadOptions,
    ) -> Result<ifccad_convert::IfccadToCadOutcome, ifccad_convert::IfccadConversionError> {
        match self {
            Self::Ifcx(value) => ifccad_convert::ifccad_source_to_cad_document(value, options),
            Self::Binary(value) => {
                ifccad_convert::ifccad_document_to_cad_document(value.document(), options)
            }
        }
    }
}

pub(crate) fn encoding(name: &str) -> &'static str {
    let name = name.to_ascii_lowercase();
    if name.ends_with(".zst") {
        "cbor-zstd"
    } else if name.ends_with(".cbor") {
        "cbor"
    } else {
        "ifcx"
    }
}

pub(crate) fn stem(name: &str) -> String {
    let base = std::path::Path::new(name)
        .file_name()
        .unwrap_or_default()
        .to_string_lossy();
    let lower = base.to_ascii_lowercase();
    [
        ".ifccad.cbor.zst",
        ".cbor.zst",
        ".ifccad.cbor",
        ".cbor",
        ".zst",
        ".ifcx.json",
        ".ifcx",
        ".json",
        ".dxf",
        ".dwg",
    ]
    .into_iter()
    .find_map(|suffix| {
        lower
            .ends_with(suffix)
            .then(|| base[..base.len() - suffix.len()].to_owned())
    })
    .unwrap_or_else(|| base.into_owned())
}

pub(crate) fn load(name: &str, bytes: &[u8]) -> Result<IfccadInput, String> {
    let cbor = ifccad::IfccadCborReadOptions {
        limits: ifccad::IfccadCborReadLimits {
            max_input_bytes: 64 * 1024 * 1024,
            ..Default::default()
        },
    };
    match encoding(name) {
        "cbor" => ifccad::load_ifccad_cbor_bytes(bytes, cbor)
            .map(IfccadInput::Binary)
            .map_err(|e| e.to_string()),
        "cbor-zstd" => load_zstd(bytes, cbor).map(IfccadInput::Binary),
        _ => ifccad::load_ifccad_bytes(bytes, Default::default())
            .map(IfccadInput::Ifcx)
            .map_err(|e| e.to_string()),
    }
}

#[cfg(feature = "ifccad-cbor-zstd")]
fn load_zstd(
    bytes: &[u8],
    cbor: ifccad::IfccadCborReadOptions,
) -> Result<ValidatedIfccadCbor, String> {
    ifccad::load_ifccad_cbor_zstd_bytes(
        bytes,
        ifccad::IfccadCborZstdReadOptions {
            limits: ifccad::IfccadCborZstdReadLimits {
                max_compressed_bytes: 64 * 1024 * 1024,
                ..Default::default()
            },
            cbor,
        },
    )
    .map_err(|e| e.to_string())
}

#[cfg(not(feature = "ifccad-cbor-zstd"))]
fn load_zstd(_: &[u8], _: ifccad::IfccadCborReadOptions) -> Result<ValidatedIfccadCbor, String> {
    Err("Zstd opening is unavailable in this processor build".into())
}

pub(crate) fn inspect(name: &str, bytes: &[u8]) -> serde_json::Value {
    use serde_json::json;
    let mut output = crate::result(std::path::Path::new(name), "ifccad");
    output["source"]["encoding"] = encoding(name).into();
    let drawing = match load(name, bytes) {
        Ok(value) => value,
        Err(error) => {
            output["reader"]["status"] = "failed".into();
            output["validation"] = json!({"strictAvailable":false,"status":"invalid","diagnostics":[{"code":"IFCCAD_BINARY_INVALID","message":error}]});
            crate::fail(&mut output, "reading", "IFCCAD_BINARY_INVALID", error);
            return output;
        }
    };
    // This is a display projection, never an original IFCX source graph or audit.
    match ifccad::encode_ifccad_document(drawing.document()) {
        Ok(display) => {
            let graph = serde_json::from_slice(display.bytes()).expect("production JSON writer");
            crate::ifccad::present_document(&mut output, drawing.document(), &graph);
            output["source"]["representation"] = "decoded".into();
            output["nativeSourceText"] = String::from_utf8(display.into_bytes())
                .expect("JSON UTF-8")
                .into();
        }
        Err(error) => crate::fail(
            &mut output,
            "presenting",
            "IFCCAD_REPRESENTATION_FAILED",
            error,
        ),
    }
    output
}
