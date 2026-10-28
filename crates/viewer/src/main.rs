use base64::{engine::general_purpose::STANDARD, Engine};
use serde_json::json;
use std::path::Path;
use viewer::{document_export_ifccad, inspect_ifccad, progress};
fn main() {
    let mut args = std::env::args().collect::<Vec<_>>();
    let _drawing_format = if let Some(index) = args.iter().position(|a| a == "--drawing-format") {
        if args.iter().filter(|a| *a == "--drawing-format").count() != 1 {
            eprintln!("Choose one drawing format");
            std::process::exit(2);
        }
        let value = args
            .get(index + 1)
            .filter(|v| v.as_str() == "ifccad")
            .cloned()
            .unwrap_or_else(|| {
                eprintln!("OCDRAW_PARKED: --drawing-format only supports ifccad");
                std::process::exit(2)
            });
        args.drain(index..index + 2);
        value
    } else {
        "ifccad".into()
    };
    let preserve_splines = args.iter().any(|arg| arg == "--preserve-splines");
    args.retain(|arg| arg != "--preserve-splines");
    let options = json!({"preserveSplines":preserve_splines}).to_string();
    let timestamp = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp");
    progress("reading");
    let result = match args.get(1).map(String::as_str) {
        Some("drawing" | "ifccad") if args.len() == 3 => inspect_ifccad(Path::new(&args[2])),
        Some("cad") if args.len() == 3 => {
            let path = Path::new(&args[2]);
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            match std::fs::read(path) {
                Ok(bytes) => {
                    viewer::document_cad_to_ifccad(&args[2], &format, &bytes, &timestamp, &options)
                }
                Err(e) => {
                    let mut r = viewer::result(path, &format);
                    viewer::fail(&mut r, "reading", "CAD_OPEN_FAILED", e);
                    r
                }
            }
        }
        Some("export-cad") if matches!(args.len(), 4 | 5) => {
            let path = Path::new(&args[2]);
            let format = path
                .extension()
                .and_then(|e| e.to_str())
                .unwrap_or("")
                .to_ascii_lowercase();
            match std::fs::read(path) {
                Ok(bytes) => {
                    let opening = viewer::document_cad_to_ifccad(
                        &args[2], &format, &bytes, &timestamp, &options,
                    );
                    if !opening["failure"].is_null() {
                        opening
                    } else {
                        let native = STANDARD
                            .decode(
                                opening["export"]["download"]["base64"]
                                    .as_str()
                                    .expect("strict native bytes"),
                            )
                            .expect("native download Base64");
                        let mut exported = viewer::document_export_ifccad(
                            &args[2],
                            &native,
                            &args[3],
                            args.get(4).map(String::as_str).unwrap_or("AC1032"),
                            &options,
                        );
                        let restoration = exported["conversion"].clone();
                        exported["conversion"] = opening["conversion"].clone();
                        exported["conversion"]["restoration"] = restoration;
                        exported["source"] = opening["source"].clone();
                        exported["reader"] = opening["reader"].clone();
                        exported
                    }
                }
                Err(e) => {
                    let mut r = viewer::result(path, &format);
                    viewer::fail(&mut r, "reading", "CAD_OPEN_FAILED", e);
                    r
                }
            }
        }
        Some("export-drawing" | "export-ifccad") if matches!(args.len(), 4 | 5) => {
            let path = Path::new(&args[2]);
            match std::fs::read(path) {
                Ok(bytes) => document_export_ifccad(
                    &args[2],
                    &bytes,
                    &args[3],
                    args.get(4).map(String::as_str).unwrap_or("AC1032"),
                    &options,
                ),
                Err(e) => {
                    let mut r = viewer::result(path, "ifccad");
                    viewer::fail(&mut r, "reading", "DRAWING_OPEN_FAILED", e);
                    r
                }
            }
        }
        _ => {
            eprintln!(
                "Usage: viewer drawing FILE | cad FILE [--drawing-format ifccad] | export-cad FILE FORMAT [VERSION] [--drawing-format ifccad] | export-drawing FILE FORMAT [VERSION]"
            );
            std::process::exit(2)
        }
    };
    println!(
        "{}",
        json!({"protocolVersion":1,"type":"result","result":result})
    );
}
