//! Manual, selected-file size probe; no timing or automatic measurement gate.
#[path = "cbor_baseline/inventory.rs"]
mod inventory;
use ifccad_convert::*;
use ocdraw::ifccad::*;
use opencadcodec::{DwgReader, DwgWriter, DxfReader, DxfWriter};
use serde_json::{json, Value};
use std::{
    fs,
    io::{Cursor, Write},
    path::Path,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
fn save(path: &Path, bytes: &[u8]) -> Result<()> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?
        .write_all(bytes)?;
    Ok(())
}
fn diagnostics<T: std::fmt::Debug>(values: T) -> Value {
    json!(format!("{values:#?}"))
}
fn main() -> Result<()> {
    let args = std::env::args_os().skip(1).collect::<Vec<_>>();
    if args.len() != 2 {
        return Err("expected INPUT.ifccad.cbor FRESH_OUTPUT_DIRECTORY".into());
    }
    let directory = Path::new(&args[1]);
    fs::create_dir(directory)?;
    let loaded = load_ifccad_cbor_file(&args[0], Default::default())?;
    let native = loaded.document();
    let target = ifccad_document_to_cad_document(native, Default::default())?;
    let before = inventory::cad(target.document());
    let mut results = Vec::new();
    for dwg in [true, false] {
        let extension = if dwg { "dwg" } else { "dxf" };
        let bytes = if dwg {
            DwgWriter::write_to_vec(target.document())?
        } else {
            DxfWriter::new(target.document()).write_to_vec()?
        };
        save(
            &directory.join(format!("foundation-roundtrip.{extension}")),
            &bytes,
        )?;
        let cad = if dwg {
            DwgReader::from_stream(Cursor::new(&bytes)).read()?
        } else {
            DxfReader::from_reader(Cursor::new(bytes.clone()))?.read()?
        };
        let after = inventory::cad(&cad);
        let imported = cad_document_to_ifccad_document(
            &cad,
            IfccadTargetMetadata {
                header: native.header.clone(),
                drawing_id: native.drawing_id,
            },
            CadToIfccadOptions {
                preservation: IfccadPreservationCapture::SupportedTyped,
                ..Default::default()
            },
        );
        let readback = match imported {
            Ok(value) => {
                validate_ifccad_document(value.document())?;
                json!({"status":"strict-valid", "inventory":inventory::native(value.document()),
                    "diagnostics":diagnostics(value.diagnostics())})
            }
            Err(error) => {
                json!({"status":"failed", "error":error.to_string(), "detail":format!("{error:?}")})
            }
        };
        println!(
            "{extension}: {} bytes; CAD reader passed; IFCCAD readback {}",
            bytes.len(),
            readback["status"]
        );
        results.push(
            json!({"format":extension,"bytes":bytes.len(),"productionCadReadback":"passed",
            "cadInventory":after,"cadInventoryEqual":before==after,"ifccadReadback":readback}),
        );
    }
    save(
        &directory.join("roundtrip.json"),
        &serde_json::to_vec_pretty(&json!({
            "scope":"one foundation file, current CBOR -> CAD -> DWG/DXF; size only",
            "nativeInventory":inventory::native(native),"targetCadInventory":before,
            "toCadDiagnostics":diagnostics(target.diagnostics()),"results":results,
            "semanticEquality":"not certified by inventory equality; diagnostics retained",
        }))?,
    )?;
    Ok(())
}
